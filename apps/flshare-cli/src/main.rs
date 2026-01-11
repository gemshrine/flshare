use std::path::PathBuf;
use clap::Parser;
use watcher::Watcher;

#[derive(Parser)]
struct Cli {
    root: PathBuf,
}

#[tokio::main]
async fn main() -> notify::Result<()> {
    let cli = Cli::parse();

    let watcher = Watcher::new(cli.root);
    watcher.run().await
}
