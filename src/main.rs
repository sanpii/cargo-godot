#![warn(warnings)]

mod opt;

use clap::Parser;
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
}

fn main() -> Result {
    let Opt::Godot(command) = Opt::parse();

    match command {
        opt::Command::Editor(args) => editor(args),
        opt::Command::Export(args) => export(args),
        opt::Command::Run(args) => run(args),
    }
}

#[derive(Debug, serde::Deserialize)]
struct Configuration {
    #[serde(default)]
    name: String,
    project: std::path::PathBuf,
    scene: Option<String>,
    remote_debug: Option<String>,
}

impl Configuration {
    fn try_from(manifest_path: &std::path::Path) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&manifest_path)
            .exec()?;

        let root = root(&metadata).unwrap();
        let package = metadata.packages.iter().find(|x| &x.id == root).unwrap();

        let mut configuration: Self = serde_json::from_value(package.metadata["godot"].clone())?;
        configuration.project = manifest_path.parent().unwrap().join(&configuration.project);
        if configuration.name.is_empty() {
            configuration.name = package.name.clone();
        }

        Ok(configuration)
    }

    fn into_args(self) -> Vec<String> {
        // --editor-pid 11043 --position 520,355
        let mut args = vec![
            "--path".to_string(),
            self.project.to_str().unwrap().to_string(),
        ];

        if let Some(remote_debug) = self.remote_debug {
            args.push("--remote-debug".to_string());
            args.push(remote_debug);
        }

        if let Some(scene) = self.scene {
            args.push(scene);
        }

        args
    }
}

fn editor(opt: opt::EditorOpt) -> Result {
    let configuration = Configuration::try_from(&opt.manifest_path)?;

    exec(
        "/usr/bin/godot",
        [
            "--editor",
            "--path",
            configuration.project.to_str().unwrap(),
        ],
    )
}

fn export(opt: opt::ExportOpt) -> Result {
    let build_mode = if opt.release {
        BuildMode::Release
    } else {
        BuildMode::Debug
    };
    build(&opt.manifest_path, build_mode)?;

    let configuration = Configuration::try_from(&opt.manifest_path)?;

    let mut path = opt.path.unwrap_or_else(|| std::path::PathBuf::from(&configuration.name));
    if !path.is_absolute() {
        path = std::env::current_dir()?.join(path);
    }

    let mut args = configuration.into_args();
    if opt.release {
        args.push("--export-release".to_string());
    } else {
        args.push("--export-debug".to_string());
    }

    args.push(opt.preset);
    args.push(path.to_str().unwrap().to_string());

    exec("/usr/bin/godot", &args)?;

    Ok(())
}

fn run(opt: opt::RunOpt) -> Result {
    build(&opt.manifest_path, BuildMode::Debug)?;

    let configuration = Configuration::try_from(&opt.manifest_path)?;
    exec("/usr/bin/godot", &configuration.into_args())?;

    Ok(())
}

fn root(metadata: &cargo_metadata::Metadata) -> Option<&cargo_metadata::PackageId> {
    metadata.resolve.as_ref()?.root.as_ref()
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

    exec("/usr/bin/cargo", &args)
}

fn exec<I, S>(program: &str, args: I) -> Result
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut child = std::process::Command::new(program).args(args).spawn()?;

    if child.wait()?.success() {
        Ok(())
    } else {
        Err(Error::Exec(program.to_string()))
    }
}
