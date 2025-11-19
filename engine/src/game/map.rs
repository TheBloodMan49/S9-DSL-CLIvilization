use std::fmt::{Display, Write};
use crate::game::utils::hash_tmb;
use noise::{NoiseFn, Perlin};

#[derive(Clone, Debug)]
pub enum Terrain {
    Water,
    Plains,
    Desert,
    City,
    Mountain,
}

pub enum TileDisplay {
    Single(&'static str, ratatui::style::Color),
    Multi(Vec<Vec<(&'static str, ratatui::style::Color)>>),
}

impl Terrain {
    pub fn to_style(&self, zoom_level: u8) -> TileDisplay {
        match self {
            Terrain::Water => TileDisplay::Single("▄", ratatui::style::Color::Indexed(26)),
            Terrain::Plains => TileDisplay::Single("▄", ratatui::style::Color::Indexed(70)),
            Terrain::Desert => TileDisplay::Single("▄", ratatui::style::Color::Indexed(220)),
            Terrain::Mountain => TileDisplay::Single("▄", ratatui::style::Color::Indexed(250)),
            Terrain::City => {
                match zoom_level {
                    1 => TileDisplay::Single("▄", ratatui::style::Color::Indexed(124)),
                    2 => {
                        // 2x2 house avec toit triangulaire
                        TileDisplay::Multi(vec![
                            vec![
                                ("◢", ratatui::style::Color::Indexed(196)), // toit gauche
                                ("◣", ratatui::style::Color::Indexed(196)), // toit droit
                            ],
                            vec![
                                ("█", ratatui::style::Color::Indexed(124)), // mur gauche
                                ("█", ratatui::style::Color::Indexed(124)), // mur droit
                            ],
                        ])
                    }
                    3 => {
                        // 3x3 house avec toit plus élaboré
                        TileDisplay::Multi(vec![
                            vec![
                                ("◢", ratatui::style::Color::Indexed(196)), // toit gauche
                                ("█", ratatui::style::Color::Indexed(196)), // toit pointe
                                ("◣", ratatui::style::Color::Indexed(196)), // toit droit
                            ],
                            vec![
                                ("█", ratatui::style::Color::Indexed(124)), // mur gauche
                                ("▓", ratatui::style::Color::Indexed(202)), // cheminée
                                ("█", ratatui::style::Color::Indexed(124)), // mur droit
                            ],
                            vec![
                                ("█", ratatui::style::Color::Indexed(124)), // mur gauche
                                ("▒", ratatui::style::Color::Indexed(220)), // porte
                                ("█", ratatui::style::Color::Indexed(124)), // mur droit
                            ],
                        ])
                    }
                    _ => TileDisplay::Single("▄", ratatui::style::Color::Indexed(124)),
                }
            }
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Terrain::Water => '~',
            Terrain::Plains => '.',
            Terrain::Desert => ':',
            Terrain::City => 'C',
            Terrain::Mountain => '^',
        }
    }
}

//todo: add sprite handling
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
