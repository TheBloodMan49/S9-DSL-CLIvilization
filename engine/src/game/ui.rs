use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
};
use super::state::GameState;

pub fn draw_ui(frame: &mut Frame, state: &GameState) {
    let size = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    draw_status_bar(frame, chunks[0], state);
    draw_main_area(frame, chunks[1], state);
    draw_resources_bar(frame, chunks[2], state);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, state: &GameState) {
    let status = Block::default()
        .title(format!("Civilization {} BC - Turn {} (Press 'q' to quit)", state.year.abs(), state.turn))
        .borders(Borders::ALL);
    frame.render_widget(status, area);
}

fn draw_main_area(frame: &mut Frame, area: Rect, state: &GameState) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ])
        .split(area);

    draw_map(frame, areas[0], state);
    draw_info_panel(frame, areas[1], state);
}

fn draw_map(frame: &mut Frame, area: Rect, state: &GameState) {
    let map_lines: Vec<Line> = state.map.tiles.iter().map(|row| {
        let spans: Vec<Span> = row.iter().map(|terrain| {
            let (color, symbol) = terrain.to_style();
            Span::styled(symbol, Style::default().fg(color).bg(color))
        }).collect();
        Line::from(spans)
    }).collect();

    let map_widget = Paragraph::new(map_lines)
        .block(Block::default().title("Map").borders(Borders::ALL));
    frame.render_widget(map_widget, area);
}

fn draw_info_panel(frame: &mut Frame, area: Rect, state: &GameState) {
    let total_population: i32 = state.cities.iter().map(|city| city.population).sum();
    let info = Paragraph::new(format!(
        "Units: 1 Settler\nCities: {}\nPopulation: {}",
        state.cities.len(),
        total_population
    ))
    .block(Block::default().title("Info").borders(Borders::ALL));
    frame.render_widget(info, area);
}

fn draw_resources_bar(frame: &mut Frame, area: Rect, state: &GameState) {
    let resources = Paragraph::new(format!(
        "Gold: {}  Science: {}  Culture: {}",
        state.resources.gold,
        state.resources.science,
        state.resources.culture
    ))
    .block(Block::default().title("Resources").borders(Borders::ALL));
    frame.render_widget(resources, area);
}
