use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "nirikeys",
    version,
    about = "Un gestor de atajos de teclado avanzado para Niri en Linux con TUI 🔑",
    long_about = None
)]
pub struct Args {
    /// Ruta personalizada al archivo de configuración de Niri.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Ejecutar en modo dry-run (no realiza cambios físicos en disco).
    #[arg(short, long)]
    pub dry_run: bool,
}
