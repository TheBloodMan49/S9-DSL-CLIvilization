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
    if let Some(buffer) = &state.map_buffer_cache {
        buffer.clone()
    } else {
        let mut map_buffer: Vec<Vec<Color>> = state
            .map
            .tiles
            .iter()
            .map(|line| line.iter().map(Terrain::to_style).collect())
            .collect();

        apply_cities_on_map_buffer(state, &mut map_buffer);

        map_buffer
    }
}

pub fn apply_cities_on_map_buffer(state: &GameState, buffer: &mut [Vec<Color>]) {
    for civ in &state.civilizations {
        let city = &civ.city;

        buffer[city.y as usize][city.x as usize] = str_to_color(&city.color);
    }
}

pub fn render_buffer<'a>(state: &GameState, area: Rect, buffer: &[Vec<Color>]) -> Vec<Line<'a>> {
    let zoom = state.zoom_level as usize;

    let visible_width = ((area.width as usize).saturating_sub(2) / zoom).min(state.map.width);
    let visible_height =
        (((area.height * 2) as usize).saturating_sub(2) / zoom).min(state.map.height);

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

pub fn draw_map(frame: &mut Frame, area: Rect, state: &GameState, ui_config: &UiConfig) {
    let buffer = generate_map_buffer(state);

    let title = if state.camera_mode {
        format!(
            "Map (Camera Mode - Position: {},{} - Zoom: {}x) - Press 'v' or Esc to exit",
            state.camera_x, state.camera_y, state.zoom_level
        )
    } else {
        format!(
            "Map (Press 'v' for camera, 'z' to zoom - Zoom: {}x)",
            state.zoom_level
        )
    };

    let buffer = generate_map_buffer(state);
    let map_lines = render_buffer(state, area, &buffer);

    // apply ui_config.color to the map widget border
    let map_widget = Paragraph::new(map_lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ui_config.color)),
    );
    frame.render_widget(map_widget, area);
}
