use noise::{NoiseFn, Perlin};
use crate::game::utils::hash_tmb;

#[derive(Clone)]
pub enum Terrain {
    Water,
    Plains,
    Desert,
    City,
    Mountain
}

impl Terrain {
    pub fn to_style(&self) -> (ratatui::style::Color, &'static str) {
        match self {
            Terrain::Water => (ratatui::style::Color::Blue, "▄"),
            Terrain::Plains => (ratatui::style::Color::Green, "▄"),
            Terrain::Desert => (ratatui::style::Color::Yellow, "▄"),
            Terrain::City => (ratatui::style::Color::Red, "▄"),
            Terrain::Mountain => (ratatui::style::Color::Gray, "▄"),
        }
    }
}

pub struct GameMap {
    pub tiles: Vec<Vec<Terrain>>,
    pub width: usize,
    pub height: usize,
    pub seed: String,
}

impl GameMap {
    pub fn new(seed: String) -> Self {
        let width = 80;
        let height = 40;
        let mut tiles = vec![vec![Terrain::Water; width]; height];

        let perlin_elevation = Perlin::new(hash_tmb(seed.clone()));
        let perlin_moisture = Perlin::new(hash_tmb(hash_tmb(seed.clone()).to_string()));
        let scale = 0.1;

        for y in 0..height {
            for x in 0..width {
                let elevation = perlin_elevation.get([x as f64 * scale, y as f64 * scale]);
                let moisture = perlin_moisture.get([x as f64 * scale * 1.5, y as f64 * scale * 1.5]);

                tiles[y][x] = match (elevation, moisture) {
                    (e, _) if e < -0.2 => Terrain::Water,
                    (e, m) if (-0.2..0.3).contains(&e) && m < -0.5 => Terrain::Desert,
                    (e, m) if (-0.2..0.3).contains(&e) && m >= -0.5 => Terrain::Plains,
                    (e, m) if e >= 0.3 && m < -0.4 => Terrain::Desert,
                    (e, _) if e >= 0.5 => Terrain::Mountain,
                    _ => Terrain::Plains,
                };
            }
        }

        // Add some cities
        tiles[10][20] = Terrain::City;
        tiles[25][50] = Terrain::City;
        tiles[30][60] = Terrain::City;

        Self {
            tiles,
            width,
            height,
            seed
        }
    }
}
