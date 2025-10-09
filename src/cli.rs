use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "yeth")]
#[command(about = "A utility for building dependency graphs between applications", long_about = None)]
pub struct Cli {
    /// Root directory to search for applications
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Name of specific application to output hash for (defaults to all)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Show only hash without application name (works only with --app)
    #[arg(short = 'H', long, requires = "app")]
    pub hash_only: bool,

    /// Show execution time statistics
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Show dependency graph
    #[arg(short = 'g', long)]
    pub show_graph: bool,

    /// Save each application's hash to yeth.version next to yeth.toml
    #[arg(short = 'w', long)]
    pub write_versions: bool,

    /// Short hash mode
    #[arg(short = 's', long)]
    pub short_hash: bool,

    /// Short hash length
    #[arg(short = 'l', long, default_value = "10")]
    pub short_hash_length: usize,
}

