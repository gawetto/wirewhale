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
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

mod l3data;
pub mod packet;
pub mod pcap;
use packet::{read_packet, Packet};
use pcap::read_pcap_header;

fn get_items_bounds(
    len: usize,
    offset: usize,
    selected: Option<usize>,
    max_height: usize,
) -> usize {
    if len == 0 {
        return 0;
    }
    let select = match selected {
        None => {
            return if len > max_height {
                len - max_height
            } else {
                0
            };
        }
        Some(x) => x,
    };
    if select < offset {
        return select;
    }
    if offset <= select && select <= (offset + max_height - 1) {
        return offset;
    }
    if select > max_height {
        select - max_height + 1
    } else {
        0
    }
}

struct StatefulList<T> {
    select: Option<usize>,
    offset: usize,
    items: Vec<T>,
    view: Option<usize>,
}

impl<T> StatefulList<T> {
    fn new() -> Self {
        Self {
            select: None,
            offset: 0,
            items: vec![],
            view: None,
        }
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.select {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => self.items.len() - 1,
        };
        self.select = Some(i);
    }

    fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.select {
            Some(i) => {
                if i == 0 {
                    i
                } else {
                    i - 1
                }
            }
            None => self.items.len() - 1,
        };
        self.select = Some(i);
    }

    fn select(&mut self) {
        if self.items.is_empty() {
            return;
        }
        match self.select {
            Some(i) => self.view = Some(i),
            None => self.select = Some(self.items.len() - 1),
        };
    }

    fn unselect(&mut self) {
        self.select = None;
    }
}

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
    packet_list: Arc<Mutex<StatefulList<Packet>>>,
    mut read: impl ReadExt + Unpin + Send + 'static,
) -> JoinHandle<()> {
    task::spawn(async move {
        while let Ok(packet) = read_packet(&mut read).await {
            packet_list.lock().unwrap().items.push(packet);
        }
    })
}

fn run_view_tick(
    packet_list: Arc<Mutex<StatefulList<Packet>>>,
    write: impl Write + Send + 'static,
    running: Arc<Mutex<bool>>,
) -> JoinHandle<()> {
    task::spawn_blocking(move || {
        let mut terminal = AlternateTerminal::new(write).unwrap();
        while *running.lock().unwrap() {
            sleep(Duration::from_millis(1));
            let _ = terminal
                .terminal
                .draw(|f| ui(f, &mut packet_list.lock().unwrap()));
        }
    })
}

pub async fn run_app<T: ReadExt + Unpin + Send + 'static, U: Write + Send + 'static>(
    mut read: T,
    write: U,
    mut event_stream: EventStream,
) -> Result<()> {
    read_pcap_header(&mut read).await?;
    let list = Arc::new(Mutex::new(StatefulList::<Packet>::new()));
    let running = Arc::new(Mutex::new(true));
    let read_packets_handle = run_read_packets(Arc::clone(&list), read);
    let view_tick_handle = run_view_tick(Arc::clone(&list), write, Arc::clone(&running));
    while let Some(Ok(event)) = event_stream.next().fuse().await {
        if let Event::Key(key) = event {
            let mut list = list.lock().unwrap();
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Left => {
                    list.unselect();
                }
                KeyCode::Down => {
                    list.next();
                }
                KeyCode::Up => {
                    list.previous();
                }
                KeyCode::Right => {
                    list.select();
                }
                _ => {}
            }
        }
    }
    *running.lock().unwrap() = false;
    read_packets_handle.cancel().await;
    view_tick_handle.await;
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, list: &mut StatefulList<Packet>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let start = get_items_bounds(
        list.items.len(),
        list.offset,
        list.select,
        chunks[0].height as usize - 2,
    );
    list.offset = start;
    let items: Vec<ListItem> = list
        .items
        .iter()
        .skip(start)
        .take(chunks[0].height as usize)
        .map(|i| {
            let lines = vec![Spans::from(i.line())];
            ListItem::new(lines)
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");
    let view = match list.view {
        Some(x) => list.items[x].text().join("\n"),
        None => "".to_string(),
    };
    let text = Paragraph::new(Text::raw(view));

    let mut state = ListState::default();
    match list.select {
        None => state.select(None),
        Some(x) => state.select(Some(x - list.offset)),
    }

    f.render_stateful_widget(items, chunks[0], &mut state);
    f.render_widget(text, chunks[1]);
}
