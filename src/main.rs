mod cli;
mod repo;
mod traits;

use clap::Parser;
use crate::cli::{Cli, Commands};
use crate::repo::repository::Repository;

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => { let _ = Repository::init(path); }
        Commands::CatFile { .. } => {}
        Commands::HashObject { .. } => {}
        Commands::LsTree { .. } => {}
        Commands::Add { .. } => {}
    }

    Ok(())
}