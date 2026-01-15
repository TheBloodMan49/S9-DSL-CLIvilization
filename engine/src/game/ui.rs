use super::state::GameState;
use crate::game::map::draw_map;
use crate::game::utils::hsv_to_rgb;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, Borders, Paragraph},
};

pub struct UiConfig {
    pub color: Color,
}

pub fn draw_ui(frame: &mut Frame, state: &mut GameState, ui_config: &UiConfig) {
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

fn draw_main_area(frame: &mut Frame, area: Rect, state: &mut GameState, ui_config: &UiConfig) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    draw_map(frame, areas[0], state, ui_config);
    draw_info_panel(frame, areas[1], state, ui_config);
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
        state
            .civilizations
            .iter()
            .map(|c| format!("- {} ({:?})", c.city.name, c.city.player_type))
            .collect::<Vec<_>>()
            .join("\n"),
        state.civilizations[state.player_turn].city.name
    );

    let info = Paragraph::new(info_text).block(
        Block::default()
            .title("Info")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color)),
    );
    frame.render_widget(info, areas[0]);

    // Player info
    // Build a string describing current constructions (or "Aucun")
    let constructions_text = if state.civilizations[state.player_turn]
        .constructions
        .is_empty()
    {
        "Aucun".to_string()
    } else {
        state.civilizations[state.player_turn]
            .constructions
            .iter()
            .map(|construction| {
                let building_name = state
                    .buildings
                    .iter()
                    .find(|u| u.name == construction.id_building)
                    .map(|u| u.name.clone())
                    .unwrap_or(construction.id_building.clone());
                format!(
                    "- {} ({} tours restants)",
                    building_name, construction.remaining
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let recruitement_text = if state.civilizations[state.player_turn]
        .recruitments
        .is_empty()
    {
        "Aucun".to_string()
    } else {
        state.civilizations[state.player_turn]
            .recruitments
            .iter()
            .map(|recruitement| {
                let building_name = state
                    .units
                    .iter()
                    .find(|u| u.name == recruitement.id_unit)
                    .map(|u| u.name.clone())
                    .unwrap_or(recruitement.id_unit.clone());
                format!(
                    "- {} ({} tours restants)",
                    building_name, recruitement.remaining
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let player_text = format!(
        "Ressources: {}\nForce Millitaire: {}\nBatiments: {}\nUnités: {}\n\nActions disponibles:\n{}\n\nBatiment en construction: \n{}\n\nUnités en recrutement: \n{}",
        state.civilizations[state.player_turn].resources.ressources,
        state.calculate_city_power(state.player_turn),
        state.civilizations[state.player_turn]
            .city
            .buildings
            .elements
            .len()
            .to_string()
            + "/"
            + &state.civilizations[state.player_turn]
                .city
                .nb_slots_buildings
                .to_string()
            + "\n- "
            + &state.civilizations[state.player_turn]
                .constructions
                .len()
                .to_string()
            + " en construction",
        0,
        "- Construire Batiment (build)\n- Recruter Unité(hire)\n- Attaquer (attack)\n- Finir Tour (end)",
        constructions_text,
        recruitement_text
    );

    let player = Paragraph::new(player_text).block(
        Block::default()
            .title(format!(
                "Jouer - {}",
                state.civilizations[state.player_turn].city.name
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color)),
    );
    frame.render_widget(player, areas[1]);
}

fn draw_action(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    // Show AI thinking message if AI is processing
    let action_text = if state.ai_thinking {
        "⏳ AI is thinking, please wait...".to_string()
    } else if state.action_editing {
        format!("{}_", state.action_input)
    } else if !state.action_input.is_empty() {
        state.action_input.clone()
    } else {
        "(press 'a' to type an action)".to_string()
    };

    let resources = Paragraph::new(action_text.clone()).block(
        Block::default()
            .title("Action")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color)),
    );
    frame.render_widget(resources, area);

    // If a popup is open, render a centered overlay on top of everything
    if let Some(popup) = &state.popup {
        let full = frame.area();
        let w = full.width.saturating_sub(10).min(60);
        let h = full.height.saturating_sub(8).min(12);
        let x = full.x + (full.width.saturating_sub(w) / 2);
        let y = full.y + (full.height.saturating_sub(h) / 2);
        let popup_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        // Build lines: prompt, choices, input
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(Span::raw(popup.prompt.clone())));
        lines.push(Line::from(Span::raw("")));
        for (i, choice) in popup.choices.iter().enumerate() {
            lines.push(Line::from(Span::raw(format!("{}. {}", i + 1, choice))));
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
            let span = Span::styled(text, Style::default().bg(Color::Black).fg(Color::White));
            bg_lines.push(Line::from(vec![span]));
        }
        let bg_block = Paragraph::new(bg_lines);
        frame.render_widget(bg_block, popup_area);
        // Build styled lines for popup content (force white on black so map colors don't bleed)
        let mut styled_lines: Vec<Line> = Vec::new();
        styled_lines.push(Line::from(Span::styled(
            popup.prompt.clone(),
            Style::default().fg(Color::White).bg(Color::Black),
        )));
        styled_lines.push(Line::from(Span::styled(
            String::new(),
            Style::default().fg(Color::White).bg(Color::Black),
        )));
        for (i, choice) in popup.choices.iter().enumerate() {
            let text = format!("{}. {}", i + 1, choice);
            styled_lines.push(Line::from(Span::styled(
                text,
                Style::default().fg(Color::White).bg(Color::Black),
            )));
        }
        if !popup.choices.is_empty() {
            styled_lines.push(Line::from(Span::styled(
                String::new(),
                Style::default().fg(Color::White).bg(Color::Black),
            )));
            styled_lines.push(Line::from(Span::styled(
                format!("Input: {}_", popup.input),
                Style::default().fg(Color::White).bg(Color::Black),
            )));
        }

        let popup_widget = Paragraph::new(styled_lines).block(
            Block::default()
                .title(popup.title.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ui_config.color)),
        );
        frame.render_widget(popup_widget, popup_area);
    }
}

pub fn draw_color_test_256(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    // Build 16x16 grid of indexed colors (0..=255)
    let cols = 16;
    let mut lines: Vec<Line> = Vec::new();
    for row in 0..16 {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..cols {
            let idx = (row * cols + col) as u8;
            // Each cell shows the index as 3 chars with background set to the indexed color
            let text = format!("{idx:>3}");
            let style = Style::default().bg(Color::Indexed(idx)).fg(Color::Reset);
            spans.push(Span::styled(text, style));
            // add a small spacer between cells
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }

    terminal.draw(|f| {
        let size = f.area();
        let block = Paragraph::new(lines.clone()).block(
            Block::default()
                .title("Terminal 256-color test (press any key to exit)")
                .borders(Borders::ALL),
        );
        f.render_widget(block, size);
    })?;

    Ok(())
}

pub fn draw_color_test_rgb(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    offset: u32,
) -> anyhow::Result<()> {
    // Build a cube of RGB colors with as much height as fits in terminal
    let mut lines: Vec<Line> = Vec::new();
    let height = terminal.size()?.height - 2; // leave space for borders
    // Show the color grid as a grid of hue and value
    for v in 0..height {
        let mut spans: Vec<Span> = Vec::new();
        for h in offset..=360 {
            let (r, g, b) = hsv_to_rgb(h as f32, 1.0, f32::from(v) / f32::from(height));
            let color = Color::Rgb(r, g, b);
            let text = "  "; // two spaces for better visibility
            let style = Style::default().bg(color).fg(Color::Reset);
            spans.push(Span::styled(text, style));
        }
        lines.push(Line::from(spans));
    }

    terminal.draw(|f| {
        let size = f.area();
        let block = Paragraph::new(lines.clone()).block(
            Block::default()
                .title("Terminal RGB-color test (press any key to exit)")
                .borders(Borders::ALL),
        );
        f.render_widget(block, size);
    })?;

    Ok(())
}

pub fn cleanup_term(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
