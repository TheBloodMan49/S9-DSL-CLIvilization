use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::Style,
};
use crate::game::map::TileDisplay;
use crate::game::utils::hsv_to_rgb;
use super::state::GameState;

pub struct UiConfig {
    pub color: Color
}

pub fn draw_ui(frame: &mut Frame, state: &GameState, ui_config: &UiConfig) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    draw_status_bar(frame, chunks[0], state, ui_config);
    draw_main_area(frame, chunks[1], state, ui_config);
    draw_resources_bar(frame, chunks[2], state, ui_config);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    let status = Block::default()
        .title(format!(
            "Civilization {} BC - Turn {} (Press Ctrl+Q to quit)",
            state.year.abs(),
            state.turn
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ui_config.color));
    frame.render_widget(status, area);
}

fn draw_main_area(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    draw_map(frame, areas[0], state, ui_config);
    draw_info_panel(frame, areas[1], state, ui_config);
}

fn draw_map(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    let zoom = state.zoom_level as usize;

    let visible_width = ((area.width as usize).saturating_sub(2) / zoom).min(state.map.width);
    let visible_height = ((area.height as usize).saturating_sub(2) / zoom).min(state.map.height);

    let start_x = (state.camera_x as usize).min(state.map.width.saturating_sub(visible_width));
    let start_y = (state.camera_y as usize).min(state.map.height.saturating_sub(visible_height));

    let mut map_lines: Vec<Line> = state.map.tiles
        .iter()
        .skip(start_y)
        .take(visible_height)
        .flat_map(|row| {
            (0..zoom).map(|_| {
                let spans: Vec<Span> = row
                    .iter()
                    .skip(start_x)
                    .take(visible_width)
                    .flat_map(|terrain| {
                        use crate::game::map::TileDisplay;
                        match terrain.to_style() {
                            TileDisplay::Single(symbol, color) => {
                                let style = Style::default().fg(color).bg(color);
                                (0..zoom).map(move |_| Span::styled(symbol, style)).collect::<Vec<_>>()
                            }
                        }
                    })
                    .collect();
                Line::from(spans)
            })
        })
        .collect();

    for civ in &state.civilizations {
        let city = &civ.city;

        for (y, line) in map_lines.iter_mut().enumerate() {
            for (x, span) in line.iter_mut().enumerate() {
                if (y + start_y) == (city.y as usize * zoom) && (x + start_x) == (city.x as usize * zoom) {
                    *span = {
                        let style = Style::default().fg(Color::Indexed(196)).bg(Color::Indexed(196));
                        Span::styled("█", style)
                    };
                }
            }
        }
    }

    let title = if state.camera_mode {
        format!(
            "Map (Camera Mode - Position: {},{} - Zoom: {}x) - Press 'v' or Esc to exit",
            state.camera_x, state.camera_y, state.zoom_level
        )
    } else {
        format!("Map (Press 'v' for camera, 'z' to zoom - Zoom: {}x)", state.zoom_level)
    };

    // apply ui_config.color to the map widget border
    let map_widget = Paragraph::new(map_lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ui_config.color))
        );
    frame.render_widget(map_widget, area);
}

fn draw_info_panel(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    // Build seed line with editing indicator
    let seed_line = if state.seed_editing {
        format!("Seed: {}{} (Enter to apply)", state.map.seed, "|")
    } else {
        format!("Seed: {} (press 's' to edit)", state.map.seed)
    };

    let info_text = format!(
        "Units: 1 Settler\nCities: {}\n\n{}\n",
        state.civilizations.len(),
        seed_line
    );

    let info = Paragraph::new(info_text)
        .block(Block::default()
            .title("Info")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color))
        );
    frame.render_widget(info, area);
}

fn draw_resources_bar(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    // TODO: use dynamic resources from state (current player)
    let resources = Paragraph::new(format!(
        "ressources: {}",
        0
    ))
    .block(Block::default()
        .title("Resources")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ui_config.color))
    );
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
        let size = f.area();
        let block = Paragraph::new(lines.clone())
            .block(Block::default().title("Terminal 256-color test (press any key to exit)").borders(Borders::ALL));
        f.render_widget(block, size);
    })?;

    Ok(())
}

pub fn draw_color_test_rgb(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, offset: u32) -> anyhow::Result<()> {
    // Build a cube of RGB colors with as much height as fits in terminal
    let mut lines: Vec<Line> = Vec::new();
    let height = terminal.size()?.height -2; // leave space for borders
    // Show the color grid as a grid of hue and value
    for v in 0..height {
        let mut spans: Vec<Span> = Vec::new();
        for h in offset..=360 {
            let (r, g, b) = hsv_to_rgb(h as f32, 1.0, v as f32 / height as f32);
            let color = Color::Rgb(r, g, b);
            let text = "  "; // two spaces for better visibility
            let style = Style::default().bg(color).fg(Color::Reset);
            spans.push(Span::styled(text, style));
        }
        lines.push(Line::from(spans));
    }

    terminal.draw(|f| {
        let size = f.area();
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
