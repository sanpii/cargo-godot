#[derive(Debug, clap::Parser)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Opt {
    #[command(subcommand)]
    Godot(Command),
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Debug(DebugOpt),
    Editor(EditorOpt),
    Export(ExportOpt),
    Run(RunOpt),
}

#[derive(Debug, clap::Parser)]
pub struct DebugOpt {
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct EditorOpt {
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct ExportOpt {
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
    #[arg(long, default_value_t = false)]
    pub release: bool,
    pub preset: String,
    pub path: Option<std::path::PathBuf>,
}

#[derive(Debug, clap::Parser)]
pub struct RunOpt {
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}
