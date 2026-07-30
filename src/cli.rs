use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "nirikeys",
    version,
    about = "Un gestor de atajos de teclado avanzado para Niri en Linux con TUI 🔑",
    long_about = None
)]
pub struct Args {
    /// Custom path to the Niri configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Run in dry-run mode (does not make physical changes to the disk).
    #[arg(short, long)]
    pub dry_run: bool,
}
