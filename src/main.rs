#![warn(warnings)]

mod config;
mod opt;

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
    #[error("Unable to read cargo metadata: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("Unable to find executable: {0}")]
    Which(String),
}

fn main() -> Result {
    let Opt::Godot(command) = Opt::parse();

    match command {
        opt::Command::Debug(args) => debug(args),
        opt::Command::Editor(args) => editor(args),
        opt::Command::Export(args) => export(args),
        opt::Command::Run(args) => run(args),
    }
}

fn debug(opt: opt::DebugOpt) -> Result {
    let config = Config::try_from(&opt.manifest_path)?;

    build(&opt.manifest_path, BuildMode::Debug)?;

    let godot = which("godot")?;
    let mut args = vec![godot.to_str().unwrap().to_string(), "--".to_string()];
    args.append(&mut config.into_args());
    exec("lldb", &args)?;

    Ok(())
}

fn editor(opt: opt::EditorOpt) -> Result {
    let config = Config::try_from(&opt.manifest_path)?;

    exec(
        "godot",
        ["--editor", "--path", config.project.to_str().unwrap()],
    )
}

fn export(opt: opt::ExportOpt) -> Result {
    let build_mode = if opt.release {
        BuildMode::Release
    } else {
        BuildMode::Debug
    };
    build(&opt.manifest_path, build_mode)?;

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

    let mut args = config.into_args();
    if opt.release {
        args.push("--export-release".to_string());
    } else {
        args.push("--export-debug".to_string());
    }

    args.push(opt.preset);
    args.push(path.to_str().unwrap().to_string());

    exec("godot", &args)?;

    Ok(())
}

fn run(opt: opt::RunOpt) -> Result {
    build(&opt.manifest_path, BuildMode::Debug)?;

    let config = Config::try_from(&opt.manifest_path)?;
    let mut args = config.into_args();
    if let Some(scene) = opt.scene {
        args.push(scene);
    }

    exec("godot", args)?;

    Ok(())
}

enum BuildMode {
    Debug,
    Release,
}

fn build(manifest_path: &std::path::Path, build_mode: BuildMode) -> Result {
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
