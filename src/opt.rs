#[derive(Debug, clap::Parser)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Opt {
    #[command(subcommand)]
    Godot(Command),
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Launch game via LLDB
    Debug(DebugOpt),
    /// Launch project in Godot
    Editor(EditorOpt),
    /// Export game
    Export(ExportOpt),
    /// Run game
    Run(RunOpt),
}

#[derive(Debug, clap::Parser)]
pub struct DebugOpt {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct EditorOpt {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct ExportOpt {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
    /// Build and export in release mode
    #[arg(long, default_value_t = false)]
    pub release: bool,
    /// Preset name
    pub preset: String,
    /// Output path. ./build/ by default
    pub path: Option<std::path::PathBuf>,
}

#[derive(Debug, clap::Parser)]
pub struct RunOpt {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
    /// Launch this specified scene instead of the default one
    pub scene: Option<String>,
}
