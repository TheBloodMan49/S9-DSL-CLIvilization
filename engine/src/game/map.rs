use crate::game::utils::hash_tmb;
use noise::{NoiseFn, Perlin};

#[derive(Clone)]
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
}


pub struct GameMap {
    pub tiles: Vec<Vec<Terrain>>,
    pub width: usize,
    pub height: usize,
    pub seed: String,
}

impl GameMap {
    pub fn new(seed: String) -> Self {
        let width = 160;
        let height = 40;
        let mut tiles = vec![vec![Terrain::Water; width]; height];

        let perlin_elevation = Perlin::new(hash_tmb(seed.clone()));
        let perlin_moisture = Perlin::new(hash_tmb(hash_tmb(seed.clone()).to_string()));
        let scale = 0.1;

        for y in 0..height {
            for x in 0..width {
                let elevation = perlin_elevation.get([x as f64 * scale, y as f64 * scale]);
                let moisture =
                    perlin_moisture.get([x as f64 * scale * 1.5, y as f64 * scale * 1.5]);

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

        // Place cities intelligently
        let mut cities = Vec::new();
        let mut min_distance = 20;
        let max_attempts = 1000;

        while cities.len() < 3 {
            let mut attempts = 0;
            let mut placed = false;

            while attempts < max_attempts && !placed {
                let x = hash_tmb(format!("{}-city-x-{}-{}", seed, cities.len(), attempts)) as usize
                    % width;
                let y = hash_tmb(format!("{}-city-y-{}-{}", seed, cities.len(), attempts)) as usize
                    % height;

                if matches!(tiles[y][x], Terrain::Plains) {
                    let mut valid = true;
                    for (cx, cy) in &cities {
                        let distance = ((x as i32 - cx).pow(2) + (y as i32 - cy).pow(2)) as f64;
                        if distance.sqrt() < min_distance as f64 {
                            valid = false;
                            break;
                        }
                    }

                    if valid {
                        tiles[y][x] = Terrain::City;
                        cities.push((x as i32, y as i32));
                        placed = true;
                    }
                }

                attempts += 1;
            }

            if !placed && min_distance > 5 {
                min_distance -= 1;
            }
        }

        Self {
            tiles,
            width,
            height,
            seed,
        }
    }

    pub fn new_random() -> Self {
        let seed = rand::random::<u64>().to_string();
        Self::new(seed)
    }
}
