#[derive(Debug, clap::Parser)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Opt {
    #[command(subcommand)]
    Godot(Command),
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Build extension
    #[command(alias = "b")]
    Build(BuildOpt),
    /// Create a new class
    Create(CreateOpt),
    /// Launch game via LLDB
    Debug(DebugOpt),
    /// Launch project in Godot
    Editor(EditorOpt),
    /// Export game
    Export(ExportOpt),
    /// Run game
    #[command(alias = "r")]
    Run(RunOpt),
    /// Execute a GD script
    Script(ScriptOpt),
}

#[derive(Debug, clap::Parser)]
pub struct BuildOpt {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct CreateOpt {
    #[arg(long, default_value = "Node")]
    pub class: String,
    #[arg(long, default_value = "./src/")]
    pub dir: std::path::PathBuf,
    pub name: String,
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
    #[arg(long)]
    pub editor_pid: Option<i32>,
    /// Launch this specified scene instead of the default one
    pub scene: Option<String>,
    #[arg(long, value_enum, num_args=0.., value_delimiter=',', default_missing_value="collisions,navigation")]
    pub debug: Vec<DebugType>,
}

#[derive(Clone, Debug, PartialEq, clap::ValueEnum)]
pub enum DebugType {
    Collisions,
    Navigation,
}

#[derive(Debug, clap::Parser)]
pub struct ScriptOpt {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
    /// Path to the script
    pub script: std::path::PathBuf,
}
