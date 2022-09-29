use anyhow::{bail, Result};
use crossterm::event::{Event, EventStream, KeyCode};
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{FutureExt, StreamExt};
use std::io::*;
use tokio::select;
use tokio_util::sync::CancellationToken;
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
mod read;
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
    return if select > max_height {
        select - max_height + 1
    } else {
        0
    };
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
        if self.items.len() == 0 {
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
        if self.items.len() == 0 {
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
        if self.items.len() == 0 {
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

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut rx: UnboundedReceiver<Packet>,
    mut event_stream: EventStream,
    cancel_token: CancellationToken,
) -> Result<()> {
    let mut list = StatefulList::new();
    loop {
        terminal.draw(|f| ui(f, &mut list))?;
        let event = event_stream.next().fuse();
        let recive = rx.next().fuse();
        select! {
            _ = cancel_token.cancelled() => bail!("cancelled"),
            maybe_rx = recive =>{
                match maybe_rx {
                    Some(x)=>{
                        list.items.push(x);
                    },
                    None=>(),
                }
            },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Left => {list.unselect();},
                            KeyCode::Down => {list.next();},
                            KeyCode::Up => {list.previous();},
                            KeyCode::Right => {list.select();},
                            _ => {}
                        }
                    }
                    Some(Ok(_)) => {
                        ()
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => ()
                }
            }
        };
    }
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
pub fn read_pcap<T: Read>(mut read: T, tx: &UnboundedSender<Packet>) -> Result<()> {
    if let Err(e) = read_pcap_header(&mut read) {
        bail!("read pcap header error {}", e)
    }
    loop {
        if let Ok(packet) = read_packet(&mut read) {
            tx.unbounded_send(packet)?;
        } else {
            break;
        }
    }
    Ok(())
}
