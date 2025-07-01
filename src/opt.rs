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
    Build(Build),
    /// Create a new class
    Create(Create),
    /// Launch game via LLDB
    Debug(Debug),
    /// Launch project in Godot
    Editor(Editor),
    /// Export game
    Export(Export),
    /// Create a new project in an existing directory
    Init(Init),
    /// Run game
    #[command(alias = "r")]
    Run(Run),
    /// Execute a GD script
    Script(Script),
}

#[derive(Debug, clap::Parser)]
pub struct Build {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct Create {
    #[arg(long, default_value = "Node")]
    pub class: String,
    #[arg(long, default_value = "./src/")]
    pub dir: std::path::PathBuf,
    pub name: String,
}

#[derive(Debug, clap::Parser)]
pub struct Debug {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct Editor {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
}

#[derive(Debug, clap::Parser)]
pub struct Export {
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
pub struct Init {
    /// Set the project name. Defaults to the directory name.
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Debug, clap::Parser)]
pub struct Run {
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
pub struct Script {
    /// Path to Cargo.toml
    #[arg(long, default_value = "./Cargo.toml")]
    pub manifest_path: std::path::PathBuf,
    /// Path to the script
    pub script: std::path::PathBuf,
}
