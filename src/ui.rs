use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::{app::InputMode, App};

fn list_ui<B: Backend>(
    f: &mut Frame<B>,
    app: &mut App,
    chunk: Rect,
    offset: &mut usize,
    style: Style,
) {
    let (items, select) = app.get_view_list(chunk.height - 2, offset);
    let items: Vec<ListItem> = items
        .iter()
        .map(|i| {
            let st = i.to_string();
            let lines = vec![Spans::from(st)];
            ListItem::new(lines)
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Packets"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .style(style)
        .highlight_symbol("");

    let mut state = ListState::default();
    state.select(select);
    f.render_stateful_widget(items, chunk, &mut state);
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, offset: &mut usize) {
    let _active_style = Style::default().bg(Color::Rgb(50, 50, 50));
    let _deactive_style = Style::default().bg(Color::Rgb(0, 0, 0));
    let filter_style = if let InputMode::Filter = app.get_input_mode() {
        _active_style
    } else {
        _deactive_style
    };
    let list_style = if let InputMode::List = app.get_input_mode() {
        _active_style
    } else {
        _deactive_style
    };
    let text_style = if let InputMode::View = app.get_input_mode() {
        _active_style
    } else {
        _deactive_style
    };

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
    let text = Paragraph::new(Text::raw(view)).style(text_style);
    let filter =
        Paragraph::new(Text::raw("filter:".to_string() + &app.get_filter())).style(filter_style);
    f.render_widget(filter, chunks[0]);
    list_ui(f, app, chunks[1], offset, list_style);
    f.render_widget(text, chunks[2]);
}
