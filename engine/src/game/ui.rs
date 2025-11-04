use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::Style,
};
use crate::game::utils::hsv_to_rgb;
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
    let zoom = state.zoom_level as usize;

    // Calculate visible area dimensions (in map tiles)
    let visible_width = ((area.width as usize).saturating_sub(2) / zoom).min(state.map.width);
    let visible_height = ((area.height as usize).saturating_sub(2) / zoom).min(state.map.height);

    // Calculate start position based on camera
    let start_x = (state.camera_x as usize).min(state.map.width.saturating_sub(visible_width));
    let start_y = (state.camera_y as usize).min(state.map.height.saturating_sub(visible_height));

    // Build the visible portion of the map with zoom
    let map_lines: Vec<Line> = state.map.tiles
        .iter()
        .skip(start_y)
        .take(visible_height)
        .flat_map(|row| {
            // For each row, create zoom_level lines
            (0..zoom).map(move |_| {
                let spans: Vec<Span> = row
                    .iter()
                    .skip(start_x)
                    .take(visible_width)
                    .flat_map(|terrain| {
                        let (color, symbol) = terrain.to_style();
                        let style = Style::default().fg(color).bg(color);
                        // Repeat the symbol zoom_level times horizontally
                        (0..zoom).map(move |_| Span::styled(symbol, style))
                    })
                    .collect();
                Line::from(spans)
            })
        })
        .collect();

    let title = if state.camera_mode {
        format!(
            "Map (Camera Mode - Position: {},{} - Zoom: {}x) - Press 'v' or Esc to exit",
            state.camera_x, state.camera_y, state.zoom_level
        )
    } else {
        format!("Map (Press 'v' for camera, 'z' to zoom - Zoom: {}x)", state.zoom_level)
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

pub fn draw_color_test_256(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
    // Build 16x16 grid of indexed colors (0..=255)
    let cols = 16;
    let mut lines: Vec<Line> = Vec::new();
    for row in 0..16 {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..cols {
            let idx = (row * cols + col) as u8;
            // Each cell shows the index as 3 chars with background set to the indexed color
            let text = format!("{:>3}", idx);
            let style = Style::default().bg(Color::Indexed(idx)).fg(Color::Reset);
            spans.push(Span::styled(text, style));
            // add a small spacer between cells
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }

    terminal.draw(|f| {
        let size = f.size();
        let block = Paragraph::new(lines.clone())
            .block(Block::default().title("Terminal 256-color test (press any key to exit)").borders(Borders::ALL));
        f.render_widget(block, size);
    })?;

    Ok(())
}

pub fn draw_color_test_rgb(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
    // Build a cube of RGB colors with as much height as fits in terminal
    let mut lines: Vec<Line> = Vec::new();
    let height = terminal.size()?.height - 10;
    // Show the color grid as a grid of hue and saturation, varying brightness by row
    for row in 0..height {
        let mut spans: Vec<Span> = Vec::new();
        let brightness = row as f32 / height as f32;
        for hue_step in 0..36 {
            let hue = hue_step as f32 * 10.0;
            for sat_step in 0..5 {
                let saturation = sat_step as f32 / 4.0;
                let (r, g, b) = hsv_to_rgb(hue, saturation, brightness);
                let text = " ";
                let style = Style::default().bg(Color::Rgb(r, g, b)).fg(Color::Reset);
                spans.push(Span::styled(text, style));
            }
            // add a small spacer between hue blocks
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }

    terminal.draw(|f| {
        let size = f.size();
        let block = Paragraph::new(lines.clone())
            .block(Block::default().title("Terminal RGB-color test (press any key to exit)").borders(Borders::ALL));
        f.render_widget(block, size);
    })?;

    Ok(())
}

pub fn cleanup_term(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
