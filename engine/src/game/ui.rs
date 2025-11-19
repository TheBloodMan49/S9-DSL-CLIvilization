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
    draw_action(frame, chunks[2], state, ui_config);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    let status = Block::default()
        .title(format!(
            "Civilization {} AC (Turn {}) (Press Ctrl+Q to quit)",
            state.turn * 10,
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

        // Position de la ville en tuiles relatives à la zone visible
        let city_tile_x = city.x as usize;
        let city_tile_y = city.y as usize;

        // Vérifier si la ville est dans la zone visible
        if city_tile_x >= start_x && city_tile_x < start_x + visible_width &&
            city_tile_y >= start_y && city_tile_y < start_y + visible_height {

            // Convertir la position de la tuile en position pixel dans map_lines
            let pixel_y_start = (city_tile_y - start_y) * zoom;
            let pixel_x_start = (city_tile_x - start_x) * zoom;

            // Dessiner la ville sur zoom x zoom pixels
            for dy in 0..zoom {
                if pixel_y_start + dy < map_lines.len() {
                    let line = &mut map_lines[pixel_y_start + dy];
                    for dx in 0..zoom {
                        if pixel_x_start + dx < line.spans.len() {
                            let style = Style::default().fg(Color::Indexed(196)).bg(Color::Indexed(196));
                            line.spans[pixel_x_start + dx] = Span::styled("█", style);
                        }
                    }
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
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Game Info
    let info_text = format!(
        "{}\n\nJoueurs: \n{}\n\nTour actuel: {}",
        format!("Seed: {}", state.map.seed),
        // List players
        state.civilizations
            .iter()
            .map(|c| format!("- {} ({:?})", c.city.name, c.city.player_type))
            .collect::<Vec<_>>()
            .join("\n"),
        state.civilizations[state.player_turn].city.name
    );

    let info = Paragraph::new(info_text)
        .block(Block::default()
            .title("Info")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color))
        );
    frame.render_widget(info, areas[0]);

    // Player info
    let player_text = format!(
        "Ressources: {}\nForce Millitaire: {}\nBatiments: {}\nUnités: {}\n\nActions disponibles:\n{}",
        state.civilizations[state.player_turn].resources.ressources,
        state.calculate_city_power(state.player_turn),
        state.civilizations[state.player_turn].city.buildings.elements.len().to_string()
            + "/"
            + &state.civilizations[state.player_turn].city.nb_slots_buildings.to_string(),
        0,
        "- Construire Batiment (build)\n- Recruter Unité(hire)\n- Attaquer (attack)\n- Finir Tour (end)"
    );

    let player = Paragraph::new(player_text)
        .block(Block::default()
            .title(
                format!(
                    "Jouer - {}",
                    state.civilizations[state.player_turn].city.name
                )
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color))
        );
    frame.render_widget(player, areas[1]);

}

fn draw_action(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    // Show current action input (editable)
    let action_text = if state.action_editing {
        format!("{}_", state.action_input)
    } else if !state.action_input.is_empty() {
        state.action_input.clone()
    } else {
        "(press 'a' to type an action)".to_string()
    };

    let resources = Paragraph::new(format!(
        "{}",
        action_text
    ))
    .block(Block::default()
        .title("Action")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ui_config.color))
    );
    frame.render_widget(resources, area);

    // If a popup is open, render a centered overlay on top of everything
    if let Some(popup) = &state.popup {
        let full = frame.area();
        let w = (full.width as u16).saturating_sub(10).min(60);
        let h = (full.height as u16).saturating_sub(8).min(12);
        let x = full.x + (full.width.saturating_sub(w) / 2);
        let y = full.y + (full.height.saturating_sub(h) / 2);
        let popup_area = Rect { x, y, width: w, height: h };

        // Build lines: prompt, choices, input
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(Span::raw(popup.prompt.clone())));
        lines.push(Line::from(Span::raw("")));
        for (i, choice) in popup.choices.iter().enumerate() {
            lines.push(Line::from(Span::raw(format!("{}. {}", i+1, choice))));
        }
        if !popup.choices.is_empty() {
            lines.push(Line::from(Span::raw("")));
            lines.push(Line::from(Span::raw(format!("Input: {}_", popup.input))));
        }

        // Draw a solid background for the popup to ensure it's visible above the map
        let mut bg_lines: Vec<Line> = Vec::new();
        let width_usize = popup_area.width as usize;
        for _ in 0..popup_area.height {
            let text = " ".repeat(width_usize);
            let span = Span::styled(text, Style::default().bg(Color::Black));
            bg_lines.push(Line::from(vec![span]));
        }
        let bg_block = Paragraph::new(bg_lines);
        frame.render_widget(bg_block, popup_area);

        let popup_widget = Paragraph::new(lines)
            .block(Block::default().title(popup.title.clone()).borders(Borders::ALL).border_style(Style::default().fg(ui_config.color)));
        frame.render_widget(popup_widget, popup_area);
    }
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
