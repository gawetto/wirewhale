use anyhow::Result;
use async_std::io::ReadExt;
use async_std::task::{self, JoinHandle};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, EventStream};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::{FutureExt, StreamExt};
use std::thread::sleep;
use std::{
    io::*,
    sync::{Arc, Mutex},
    time::Duration,
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub mod app;
pub mod filtable;
mod input_action;
mod l3data;
pub mod packet;
pub mod pcap;
pub mod ui;
use app::App;
use app::InputMode;
use packet::{read_packet, Packet};
use pcap::read_pcap_header;

pub struct AlternateTerminal<T: Write> {
    terminal: Terminal<CrosstermBackend<T>>,
}

impl<T: Write> AlternateTerminal<T> {
    pub fn new(write: T) -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(write))?;
        let mut ret = Self { terminal };
        execute!(ret.terminal.backend_mut(), EnterAlternateScreen, Hide)?;
        Ok(ret)
    }
}

impl<T: Write> Drop for AlternateTerminal<T> {
    fn drop(&mut self) {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, Show).unwrap();
    }
}

fn run_read_packets(
    app: Arc<Mutex<App>>,
    mut read: impl ReadExt + Unpin + Send + 'static,
) -> JoinHandle<()> {
    task::spawn(async move {
        while let Ok(packet) = read_packet(&mut read).await {
            app.lock().unwrap().add_packet(packet);
        }
    })
}

fn run_view_tick(app: Arc<Mutex<App>>, write: impl Write + Send + 'static) -> JoinHandle<()> {
    task::spawn_blocking(move || {
        let mut terminal = AlternateTerminal::new(write).expect("terminal init err");
        let mut offset = 0;
        loop {
            sleep(Duration::from_millis(100));
            let mut app = app.lock().unwrap();
            if !app.is_running() {
                break;
            }
            let _ = terminal.terminal.draw(|f| ui::ui(f, &mut app, &mut offset));
        }
    })
}

pub async fn run_app<T: ReadExt + Unpin + Send + 'static, U: Write + Send + 'static>(
    mut read: T,
    write: U,
    mut event_stream: EventStream,
) -> Result<()> {
    read_pcap_header(&mut read).await?;
    let app = Arc::new(Mutex::new(App::default()));
    let read_packets_handle = run_read_packets(Arc::clone(&app), read);
    let view_tick_handle = run_view_tick(Arc::clone(&app), write);
    while let Some(Ok(event)) = event_stream.next().fuse().await {
        let mut app = app.lock().unwrap();
        let key = if let Event::Key(key) = event {
            key
        } else {
            continue;
        };
        input_action::allmode_input(&mut app, key.code);
        eprintln!("{}", app);
        match app.get_input_mode() {
            InputMode::List => input_action::listmode_input(&mut app, key.code),
            InputMode::View => input_action::viewmode_input(&mut app, key.code),
            InputMode::Filter => input_action::filtermode_input(&mut app, key.code),
        }
        if !app.is_running() {
            break;
        }
    }
    read_packets_handle.cancel().await;
    view_tick_handle.await;
    Ok(())
}
