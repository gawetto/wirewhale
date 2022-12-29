use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::App;

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

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, offset: &mut usize) {
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
