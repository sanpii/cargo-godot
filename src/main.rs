#![warn(warnings)]

mod config;
mod opt;

use std::io::Write;

use clap::Parser;
use config::Config;
use opt::Opt;

type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("exec '{0}' failed")]
    Exec(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unable to read cargo manifest: {0}")]
    Manifest(#[from] cargo_metadata::Error),
    #[error("Missing package.metadata.godot configuration in Cargo.toml")]
    MissingMetadata,
    #[error("Unable to read cargo metadata: {0}")]
    InvalidMetadata(#[from] serde_json::Error),
    #[error("Unable to find executable: {0}")]
    Which(String),
}

fn main() -> anyhow::Result<()> {
    let Opt::Godot(command) = Opt::parse();

    match command {
        opt::Command::Build(args) => build(args)?,
        opt::Command::Create(args) => create(args)?,
        opt::Command::Debug(args) => debug(args)?,
        opt::Command::Editor(args) => editor(args)?,
        opt::Command::Export(args) => export(args)?,
        opt::Command::Init(args) => init(args)?,
        opt::Command::Run(args) => run(args)?,
        opt::Command::Script(args) => script(args)?,
    };

    Ok(())
}

fn build(opt: opt::Build) -> Result {
    cargo_build(&opt.manifest_path, BuildMode::Debug)?;

    let config = Config::try_from(&opt.manifest_path)?;
    let pkgname = config.name.clone();

    let target = pathdiff::diff_paths(
        opt.manifest_path.parent().unwrap().canonicalize().unwrap(),
        config.project,
    )
    .unwrap()
    .join("target");

    let contents = format!(
        r#"
[configuration]
entry_symbol = "gdext_rust_init"
compatibility_minimum = 4.1
reloadable = true

[libraries]
linux.debug.x86_64 =     "res://{target}/debug/lib{pkgname}.so"
linux.release.x86_64 =   "res://{target}/release/lib{pkgname}.so"
windows.debug.x86_64 =   "res://{target}/debug/{pkgname}.dll"
windows.release.x86_64 = "res://{target}/release/{pkgname}.dll"
macos.debug =            "res://{target}/debug/lib{pkgname}.dylib"
macos.release =          "res://{target}/release/lib{pkgname}.dylib"
macos.debug.arm64 =      "res://{target}/debug/lib{pkgname}.dylib"
macos.release.arm64 =    "res://{target}/release/lib{pkgname}.dylib"
"#,
        target = target.display()
    );

    let mut gdextension = opt.manifest_path.parent().unwrap().join(pkgname);
    gdextension.set_extension("gdextension");
    std::fs::write(&gdextension, contents)?;
    Ok(())
}

fn create(opt: opt::Create) -> Result {
    use convert_case::Casing;

    let contents = format!(
        r#"use godot::engine::I{class};
use godot::obj::WithBaseField as _;

#[derive(Debug, godot::register::GodotClass)]
#[class(init, base={class})]
struct {name} {{
    base: godot::obj::Base<godot::engine::{class}>,
}}

#[godot::register::godot_api]
impl I{class} for {name} {{
    fn ready(&mut self) {{
    }}

    fn process(&mut self, _delta: f64) {{
    }}
}}
"#,
        class = opt.class,
        name = opt.name
    );

    let filename = opt.name.to_case(convert_case::Case::Snake);
    let mut file = opt.dir.join(filename);
    file.set_extension("rs");

    if file.exists() {
        eprintln!("File already exists");
    } else {
        std::fs::write(&file, contents)?;
    }
    Ok(())
}

fn debug(opt: opt::Debug) -> Result {
    let config = Config::try_from(&opt.manifest_path)?;

    cargo_build(&opt.manifest_path, BuildMode::Debug)?;

    let (godot, mut args) = config.into_args();
    args.insert(0, which(&godot)?.to_str().unwrap().to_string());
    args.insert(1, "--".to_string());

    exec("lldb", &args)?;

    Ok(())
}

fn editor(opt: opt::Editor) -> Result {
    let config = Config::try_from(&opt.manifest_path)?;

    exec(
        &config.godot_executable,
        ["--editor", "--path", config.project.to_str().unwrap()],
    )
}

fn export(opt: opt::Export) -> Result {
    let build_mode = if opt.release {
        BuildMode::Release
    } else {
        BuildMode::Debug
    };
    cargo_build(&opt.manifest_path, build_mode)?;

    let config = Config::try_from(&opt.manifest_path)?;

    let mut path = opt
        .path
        .unwrap_or_else(|| std::path::PathBuf::from(format!("build/{}", config.name)));
    if !path.is_absolute() {
        path = std::env::current_dir()?.join(path);
    }

    if !path.parent().unwrap().exists() {
        std::fs::create_dir_all(path.parent().unwrap())?;
    }

    let (godot, mut args) = config.into_args();
    if opt.release {
        args.push("--export-release".to_string());
    } else {
        args.push("--export-debug".to_string());
    }

    args.push(opt.preset);
    args.push(path.to_str().unwrap().to_string());

    exec(&godot, &args)?;

    Ok(())
}

fn init(opt: opt::Init) -> Result {
    let name = if let Some(name) = opt.name {
        name
    } else {
        let cwd = std::env::current_dir()?;

        cwd.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    };

    exec("cargo", ["new", "--lib", "--name", &name, "rust"])?;
    exec(
        "cargo",
        ["add", "godot", "--manifest-path", "rust/Cargo.toml"],
    )?;

    let mut manifest = std::fs::OpenOptions::new()
        .append(true)
        .open("rust/Cargo.toml")?;

    let contents = r#"
[lib]
crate-type = ["cdylib"]

[package.metadata.godot]
project = "../godot"
"#;

    manifest.write_all(contents.as_bytes())?;

    std::fs::write(
        "rust/src/lib.rs",
        r#"use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
"#,
    )?;

    std::fs::create_dir("godot")?;
    std::fs::File::create("godot/project.godot")?;

    symlink(
        format!("../rust/{name}.gdextension"),
        format!("godot/{name}.gdextension"),
    )?;

    Ok(())
}

fn symlink<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(original: P, link: Q) -> Result {
    #[cfg(unix)]
    std::os::unix::fs::symlink(original, link)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(original, link)?;

    Ok(())
}

fn run(opt: opt::Run) -> Result {
    let build_opt = opt::Build {
        manifest_path: opt.manifest_path.clone(),
    };
    build(build_opt)?;

    let config = Config::try_from(&opt.manifest_path)?;
    let (godot, mut args) = config.into_args();

    if let Some(editor_pid) = opt.editor_pid {
        args.push("--editor-pid".to_string());
        args.push(editor_pid.to_string());
    }

    if opt.debug.contains(&opt::DebugType::Collisions) {
        args.push("--debug-collisions".to_string());
    }
    if opt.debug.contains(&opt::DebugType::Navigation) {
        args.push("--debug-navigation".to_string());
    }

    if let Some(scene) = opt.scene {
        args.push(scene);
    }

    exec(&godot, args)?;

    Ok(())
}

fn script(opt: opt::Script) -> Result {
    let config = Config::try_from(&opt.manifest_path)?;
    let (godot, mut args) = config.into_args();
    args.push("--script".to_string());
    args.push(opt.script.to_str().unwrap().to_string());

    exec(&godot, args)?;

    Ok(())
}

enum BuildMode {
    Debug,
    Release,
}

fn cargo_build(manifest_path: &std::path::Path, build_mode: BuildMode) -> Result {
    let mut args = vec!["build", "--manifest-path", manifest_path.to_str().unwrap()];

    if matches!(build_mode, BuildMode::Release) {
        args.push("--release");
    }

    exec("cargo", &args)
}

fn exec<I, S>(program: &str, args: I) -> Result
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let executable = which(program)?;
    let mut child = std::process::Command::new(executable).args(args).spawn()?;

    if child.wait()?.success() {
        Ok(())
    } else {
        Err(Error::Exec(program.to_string()))
    }
}

fn which(program: &str) -> Result<std::path::PathBuf> {
    which::which(program).map_err(|_| Error::Which(program.to_string()))
}
