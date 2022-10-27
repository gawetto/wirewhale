use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::EventStream,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use std::io::{stdin, stdout};
use tokio_util::sync::CancellationToken;
use tui::{backend::CrosstermBackend, Terminal};
use wirewhale::{packet::Packet, read_pcap, run_app};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {}

#[tokio::main]
async fn main() -> Result<()> {
    Cli::parse();
    let (tx, rx): (UnboundedSender<Packet>, UnboundedReceiver<Packet>) = unbounded();
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    tokio::spawn(async move {
        if read_pcap(stdin(), &tx).is_err() {
            token.cancel();
        }
    });

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen,)?;
    let ret = run_app(&mut terminal, rx, EventStream::new(), cloned_token).await;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    disable_raw_mode()?;
    ret
}
