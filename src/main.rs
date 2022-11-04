use anyhow::Result;
use async_std::io::stdin;
use clap::Parser;
use crossterm::event::EventStream;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::stdout;
use wirewhale::run_app;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {}

#[async_std::main]
async fn main() -> Result<()> {
    Cli::parse();
    enable_raw_mode()?;
    let ret = run_app(stdin(), stdout(), EventStream::new()).await;
    disable_raw_mode()?;
    ret
}
