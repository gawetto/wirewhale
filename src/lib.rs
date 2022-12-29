use anyhow::Result;
use async_std::io::ReadExt;
use async_std::task::{self, JoinHandle};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, EventStream, KeyCode};
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
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

pub mod app;
mod l3data;
pub mod packet;
pub mod pcap;
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

fn list_ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect, offset: &mut usize) {
    let (items, select) = app.get_view_list(chunk.height - 2, offset);
    let items: Vec<ListItem> = items
        .iter()
        .map(|i| {
            let st = i.to_string();
            let lines = vec![Spans::from(st)];
            ListItem::new(lines)
        })
        .collect();

    let brdr_style = Style::default()
        .bg(Color::Rgb(50, 50, 50))
        .fg(Color::Rgb(150, 150, 150));
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Packets"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .style(brdr_style)
        .highlight_symbol("");

    let mut state = ListState::default();
    state.select(select);
    f.render_stateful_widget(items, chunk, &mut state);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, offset: &mut usize) {
    let _active_bg = Color::Rgb(50, 50, 50);
    let _deactive_bg = Color::Rgb(0, 0, 0);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(f.size());
    let view = app.get_view_text();
    let text = Paragraph::new(Text::raw(view));
    let filter = Paragraph::new(Text::raw("filter:".to_string()));
    f.render_widget(filter, chunks[0]);
    list_ui(f, app, chunks[1], offset);
    f.render_widget(text, chunks[2]);
}

fn run_view_tick(
    app: Arc<Mutex<App>>,
    write: impl Write + Send + 'static,
    running: Arc<Mutex<bool>>,
) -> JoinHandle<()> {
    task::spawn_blocking(move || {
        let mut terminal = AlternateTerminal::new(write).expect("terminal init err");
        let mut offset = 0;
        while *running.lock().unwrap() {
            sleep(Duration::from_millis(100));
            let _ = terminal
                .terminal
                .draw(|f| ui(f, &mut app.lock().unwrap(), &mut offset));
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
    app.lock().unwrap().add_filter_char(b't');
    app.lock().unwrap().add_filter_char(b'y');
    app.lock().unwrap().add_filter_char(b'p');
    app.lock().unwrap().add_filter_char(b'e');
    let running = Arc::new(Mutex::new(true));
    let read_packets_handle = run_read_packets(Arc::clone(&app), read);
    let view_tick_handle = run_view_tick(Arc::clone(&app), write, Arc::clone(&running));
    while let Some(Ok(event)) = event_stream.next().fuse().await {
        if let Event::Key(key) = event {
            if let KeyCode::Tab = key.code {
                app.lock().unwrap().next_forcus();
                continue;
            }
            let mode = app.lock().unwrap().get_input_mode();
            match mode {
                InputMode::List => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => {
                        app.lock().unwrap().unselect();
                    }
                    KeyCode::Down => {
                        app.lock().unwrap().next();
                    }
                    KeyCode::Up => {
                        app.lock().unwrap().previous();
                    }
                    KeyCode::Right => {
                        app.lock().unwrap().to_view();
                    }
                    _ => {}
                },
                InputMode::View => {}
                InputMode::Filter => {}
            }
        }
    }
    *running.lock().unwrap() = false;
    read_packets_handle.cancel().await;
    view_tick_handle.await;
    Ok(())
}
