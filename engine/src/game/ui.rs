use super::state::GameState;
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, Borders, Paragraph},
};

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
        .title(format!(
            "Civilization {} BC - Turn {} (Press Ctrl+Q to quit)",
            state.year.abs(),
            state.turn
        ))
        .borders(Borders::ALL);
    frame.render_widget(status, area);
}

fn draw_main_area(frame: &mut Frame, area: Rect, state: &GameState) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    draw_map(frame, areas[0], state);
    draw_info_panel(frame, areas[1], state);
}

fn draw_map(frame: &mut Frame, area: Rect, state: &GameState) {
    // Calculate visible area dimensions
    let visible_width = (area.width as usize).min(state.map.width);
    let visible_height = (area.height.saturating_sub(2) as usize).min(state.map.height); // -2 for borders

    // Calculate start position based on camera
    let start_x = (state.camera_x as usize).min(state.map.width.saturating_sub(visible_width));
    let start_y = (state.camera_y as usize).min(state.map.height.saturating_sub(visible_height));

    // Build only the visible portion of the map
    let map_lines: Vec<Line> = state.map.tiles
        .iter()
        .skip(start_y)
        .take(visible_height)
        .map(|row| {
            let spans: Vec<Span> = row
                .iter()
                .skip(start_x)
                .take(visible_width)
                .map(|terrain| {
                    let (color, symbol) = terrain.to_style();
                    Span::styled(symbol, Style::default().fg(color).bg(color))
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    let title = if state.camera_mode {
        format!("Map (Camera Mode - Position: {},{}) - Press 'v' or Esc to exit", state.camera_x, state.camera_y)
    } else {
        "Map (Press 'v' for camera mode)".to_string()
    };

    let map_widget = Paragraph::new(map_lines)
        .block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(map_widget, area);
}

fn draw_info_panel(frame: &mut Frame, area: Rect, state: &GameState) {
    let total_population: i32 = state.cities.iter().map(|city| city.population).sum();

    // Build seed line with editing indicator
    let seed_line = if state.seed_editing {
        format!("Seed: {}{} (Enter to apply)", state.seed_input, "|")
    } else {
        format!("Seed: {} (press 's' to edit)", state.seed_input)
    };

    let info_text = format!(
        "Units: 1 Settler\nCities: {}\nPopulation: {}\n\n{}\n",
        state.cities.len(),
        total_population,
        seed_line
    );

    let info =
        Paragraph::new(info_text).block(Block::default().title("Info").borders(Borders::ALL));
    frame.render_widget(info, area);
}

fn draw_resources_bar(frame: &mut Frame, area: Rect, state: &GameState) {
    let resources = Paragraph::new(format!(
        "Gold: {}  Science: {}  Culture: {}",
        state.resources.gold, state.resources.science, state.resources.culture
    ))
    .block(Block::default().title("Resources").borders(Borders::ALL));
    frame.render_widget(resources, area);
}
