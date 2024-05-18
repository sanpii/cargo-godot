#![warn(warnings)]

use clap::Parser;

type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("cargo build failed")]
    Build,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unable to read cargo manifest: {0}")]
    Manifest(#[from] cargo_metadata::Error),
    #[error("Unable to read cargo metadata: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("Unable to run godot scene")]
    Run,
}

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Opt {
    #[command(subcommand)]
    Godot(Command),
}

#[derive(clap::Subcommand)]
enum Command {
    Run(Args),
}

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "./Cargo.toml")]
    manifest_path: std::path::PathBuf,
}

fn main() -> Result {
    let Opt::Godot(command) = Opt::parse();

    match command {
        Command::Run(args) => run(args),
    }
}

#[derive(Debug, serde::Deserialize)]
struct Configuration {
    project: std::path::PathBuf,
    scene: Option<String>,
    remote_debug: Option<String>,
}

impl Configuration {
    fn into_args(self) -> Vec<String> {
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

fn run(args: Args) -> Result {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&args.manifest_path)
        .exec()?;

    let root = root(&metadata).unwrap();
    let package = metadata.packages.iter().find(|x| &x.id == root).unwrap();

    let mut configuration: Configuration = serde_json::from_value(package.metadata["godot"].clone())?;
    configuration.project = args.manifest_path.parent().unwrap().join(&configuration.project);

    build(&args)?;
    exec(configuration)?;

    Ok(())
}

fn root(metadata: &cargo_metadata::Metadata) -> Option<&cargo_metadata::PackageId> {
    metadata.resolve.as_ref()?.root.as_ref()
}

fn build(args: &Args) -> Result {
    let mut child = std::process::Command::new("/usr/bin/cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&args.manifest_path)
        .spawn()?;

    if child.wait()?.success() {
        Ok(())
    } else {
        Err(Error::Build)
    }
}

fn exec(configuration: Configuration) -> Result {
    let mut child = std::process::Command::new("/usr/bin/godot")
        .args(configuration.into_args())
        .spawn()?;

    if child.wait()?.success() {
        Ok(())
    } else {
        Err(Error::Run)
    }
}
