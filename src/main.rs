#![warn(warnings)]

use clap::Parser;

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

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Opt {
    #[command(subcommand)]
    Godot(Command),
}

#[derive(clap::Subcommand)]
enum Command {
    Editor(Args),
    Run(Args),
    //Debug,
}

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "./Cargo.toml")]
    manifest_path: std::path::PathBuf,
}

fn main() -> Result {
    let Opt::Godot(command) = Opt::parse();

    match command {
        Command::Editor(args) => editor(args),
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

impl TryFrom<Args> for Configuration {
    type Error = Error;

    fn try_from(value: Args) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&value.manifest_path)
            .exec()?;

        let root = root(&metadata).unwrap();
        let package = metadata.packages.iter().find(|x| &x.id == root).unwrap();

        let mut configuration: Self = serde_json::from_value(package.metadata["godot"].clone())?;
        configuration.project = value.manifest_path.parent().unwrap().join(&configuration.project);

        Ok(configuration)
    }
}

fn editor(args: Args) -> Result {
    let configuration = Configuration::try_from(args)?;

    exec("/usr/bin/godot", [
        "--editor",
        "--path",
        configuration.project.to_str().unwrap(),
    ])
}

fn run(args: Args) -> Result {
    build(&args)?;

    let configuration = Configuration::try_from(args)?;
    exec("/usr/bin/godot", &configuration.into_args())?;

    Ok(())
}

fn root(metadata: &cargo_metadata::Metadata) -> Option<&cargo_metadata::PackageId> {
    metadata.resolve.as_ref()?.root.as_ref()
}

fn build(args: &Args) -> Result {
    exec("/usr/bin/cargo", ["build", "--manifest-path", &args.manifest_path.to_str().unwrap()])
}

fn exec<I, S>(program: &str, args: I) -> Result
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut child = std::process::Command::new(program)
        .args(args)
        .spawn()?;

    if child.wait()?.success() {
        Ok(())
    } else {
        Err(Error::Exec(program.to_string()))
    }
}
