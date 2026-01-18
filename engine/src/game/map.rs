use crate::game::state::GameState;
use crate::game::ui::UiConfig;
use crate::game::utils::{hash_tmb, str_to_color};
use noise::{NoiseFn, Perlin};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::fmt::{Display, Write};

#[derive(Clone, Debug)]
pub enum Terrain {
    Water,
    Plains,
    Desert,
    Mountain,
}

impl Terrain {
    pub fn to_style(&self) -> Color {
        match self {
            Terrain::Water => Color::Indexed(26),
            Terrain::Plains => Color::Indexed(70),
            Terrain::Desert => Color::Indexed(220),
            Terrain::Mountain => Color::Indexed(250),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Terrain::Water => '~',
            Terrain::Plains => '.',
            Terrain::Desert => ':',
            Terrain::Mountain => '^',
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameMap {
    pub tiles: Vec<Vec<Terrain>>,
    pub width: usize,
    pub height: usize,
    pub seed: String,
}

impl GameMap {
    pub fn new(seed: String, width: usize, height: usize) -> Self {
        let mut tiles = vec![vec![Terrain::Water; width]; height];

        let perlin_elevation = Perlin::new(hash_tmb(seed.clone()));
        let perlin_moisture = Perlin::new(hash_tmb(hash_tmb(seed.clone()).to_string()));
        let scale = 0.1;

        for (y, line) in tiles.iter_mut().enumerate() {
            for (x, cell) in line.iter_mut().enumerate() {
                let elevation = perlin_elevation.get([x as f64 * scale, y as f64 * scale]);
                let moisture =
                    perlin_moisture.get([x as f64 * scale * 1.5, y as f64 * scale * 1.5]);

                *cell = match (elevation, moisture) {
                    (e, _) if e < -0.2 => Terrain::Water,
                    (e, m) if (-0.2..0.3).contains(&e) && m < -0.5 => Terrain::Desert,
                    (e, m) if (-0.2..0.3).contains(&e) && m >= -0.5 => Terrain::Plains,
                    (e, m) if e >= 0.3 && m < -0.4 => Terrain::Desert,
                    (e, _) if e >= 0.5 => Terrain::Mountain,
                    _ => Terrain::Plains,
                };
            }
        }

        Self {
            tiles,
            width,
            height,
            seed,
        }
    }

    pub fn new_random(width: usize, height: usize) -> Self {
        let seed = rand::random::<u64>().to_string();
        Self::new(seed, width, height)
    }
}

impl Display for GameMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for row in &self.tiles {
            for terrain in row {
                formatter.write_char(terrain.to_char())?;
            }
            formatter.write_char('\n')?;
        }

        Ok(())
    }
}

pub fn generate_map_buffer(state: &GameState) -> Vec<Vec<Color>> {
    // Use cached terrain buffer if present, but always overlay dynamic entities (cities, travels)
    let mut base: Vec<Vec<Color>> = if let Some(buffer) = &state.map_buffer_cache {
        buffer.clone()
    } else {
        state
            .map
            .tiles
            .iter()
            .map(|line| line.iter().map(Terrain::to_style).collect())
            .collect()
    };

    apply_cities_on_map_buffer(state, &mut base);
    base
}

pub fn apply_cities_on_map_buffer(state: &GameState, buffer: &mut [Vec<Color>]) {
    for civ in &state.civilizations {
        let city = &civ.city;
        // draw city
        if (city.y as usize) < buffer.len() && (city.x as usize) < buffer[0].len() {
            buffer[city.y as usize][city.x as usize] = str_to_color(&city.color);
        }
    }

    // draw traveling units along their paths
    for t in &state.travels {
        if t.path.is_empty() { continue; }

        // optionally draw full path when zoomed in (visible when zoom > 1)
        // draw a continuous path: stop one tile before destination and don't override cities
        for (i, (sx, sy)) in t.path.iter().enumerate() {
            // stop before destination (last element)
            if i + 1 >= t.path.len() { break; }
            if *sy < 0 || *sx < 0 { continue; }
            let syu = *sy as usize;
            let sxu = *sx as usize;
            if syu >= buffer.len() || sxu >= buffer[0].len() { continue; }

            // don't overwrite city tiles
            let mut is_city = false;
            for civ in &state.civilizations {
                if civ.city.x as usize == sxu && civ.city.y as usize == syu {
                    is_city = true;
                    break;
                }
            }
            if is_city { continue; }

            // draw path tile
            buffer[syu][sxu] = Color::Indexed(8);
        }

        // compute progress index along path from travel.remaining/total
        let total_turns = t.total.max(1) as f64;
        let passed = (t.total - t.remaining) as f64;
        let fraction = (passed / total_turns).clamp(0.0, 1.0);
        let total_steps = if t.path.len() >= 1 { t.path.len() - 1 } else { 0 } as f64;
        // use floor to avoid jumping to the next tile too early
        let idx = (fraction * total_steps).floor() as usize;
        let pos = t.path.get(idx).unwrap_or(&t.path[t.path.len()-1]);
        let (px, py) = *pos;
        if py >= 0 && px >= 0 && (py as usize) < buffer.len() && (px as usize) < buffer[0].len() {
            // use attacker's color to mark traveling unit (draw on top of path)
            let col = ratatui::style::Color::Cyan;
            buffer[py as usize][px as usize] = col;
        }
    }
}

pub fn render_buffer<'a>(state: &GameState, _area: Rect, buffer: &[Vec<Color>], visible_width: usize, visible_height: usize) -> Vec<Line<'a>> {
    let zoom = state.zoom_level as usize;

    let start_x = (state.camera_x as usize).min(state.map.width.saturating_sub(visible_width));
    let start_y = (state.camera_y as usize).min(state.map.height.saturating_sub(visible_height));

    let _stop_x = start_x + visible_width;
    let stop_y = start_y + visible_height;

    buffer[start_y..stop_y]
        .iter()
        .flat_map(|t| (0..zoom).map(|_| t.clone()))
        .collect::<Vec<Vec<Color>>>()
        .chunks_exact(2)
        .map(|pair| {
            Line::from(
                pair[0]
                    .iter()
                    .zip(&pair[1])
                    .skip(start_x)
                    .take(visible_width)
                    .flat_map(|(c1, c2)| {
                        (0..zoom).map(|_| Span::styled("▄", Style::new().bg(*c1).fg(*c2)))
                    })
                    .collect::<Vec<Span>>(),
            )
        })
        .collect::<Vec<Line>>()
}

/// Draw the game map to a frame area.
///
/// Main entry point for rendering the map in the TUI.
/// Handles buffer generation, caching, camera positioning, and zoom.
///
/// # Arguments
/// * `frame` - The ratatui Frame to draw on
/// * `area` - The screen area to render into
/// * `state` - Current game state (mutable for caching)
/// * `ui_config` - UI configuration
pub fn draw_map(frame: &mut Frame, area: Rect, state: &mut GameState, ui_config: &UiConfig) {
    let visible_width = (usize::from(area.width).saturating_sub(2) / usize::from(state.zoom_level)).min(state.map.width);
    let visible_height = (usize::from(area.height * 2).saturating_sub(2) / usize::from(state.zoom_level)).min(state.map.height);

    let hidden_width = state.map.width - visible_width;
    let hidden_height = state.map.height - visible_height;

    state.camera_x = state.camera_x.clamp(
        0,
        hidden_width as i32,
    );
    state.camera_y = state.camera_y.clamp(
        0,
        hidden_height as i32,
    );

    let title = if state.camera_mode {
        format!(
            "Map (Camera Mode - Position: {}/{},{}/{} - Zoom: {}x) - Press 'v' or Esc to exit",
            state.camera_x, hidden_width, state.camera_y, hidden_height, state.zoom_level
        )
    } else {
        format!(
            "Map (Press 'v' for camera, 'z' to zoom - Zoom: {}x)",
            state.zoom_level
        )
    };

    let buffer = generate_map_buffer(state);
    let map_lines = render_buffer(state, area, &buffer, visible_width, visible_height);

    // apply ui_config.color to the map widget border
    let map_widget = Paragraph::new(map_lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color)),
    );
    frame.render_widget(map_widget, area);
}
