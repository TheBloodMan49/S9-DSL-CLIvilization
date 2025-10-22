use noise::{NoiseFn, Perlin};

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
}

impl GameMap {
    pub fn new() -> Self {
        let width = 80;
        let height = 40;
        let mut tiles = vec![vec![Terrain::Water; width]; height];

        let perlin = Perlin::new(851);
        let scale = 0.1;

        for y in 0..height {
            for x in 0..width {
                let noise_value = perlin.get([x as f64 * scale, y as f64 * scale]);

                tiles[y][x] = match noise_value {
                    v if v < -0.2 => Terrain::Water,
                    v if v < 0.5 => Terrain::Plains,
                    v if v < 0.6 => Terrain::Desert,
                    _ => Terrain::Mountain,
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
        }
    }
}
