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
        let mut tiles = vec![vec![Terrain::Water; 40]; 20];

        // First island (left)
        for y in 4..12 {
            for x in 5..15 {
                tiles[y][x] = Terrain::Plains;
            }
        }
        // Mountains on first island
        tiles[6][7] = Terrain::Mountain;
        tiles[6][8] = Terrain::Mountain;
        tiles[7][7] = Terrain::Mountain;
        // Desert area on first island
        for y in 8..11 {
            for x in 11..14 {
                tiles[y][x] = Terrain::Desert;
            }
        }

        // Second island (right)
        for y in 8..16 {
            for x in 25..35 {
                tiles[y][x] = Terrain::Plains;
            }
        }
        // Mountains on second island
        tiles[10][28] = Terrain::Mountain;
        tiles[11][28] = Terrain::Mountain;
        tiles[11][29] = Terrain::Mountain;
        tiles[12][29] = Terrain::Mountain;
        // Desert area on second island
        for y in 12..15 {
            for x in 30..33 {
                tiles[y][x] = Terrain::Desert;
            }
        }

        // Add some cities
        tiles[7][9] = Terrain::City;
        tiles[13][31] = Terrain::City;

        let height = tiles.len();
        let width = tiles[0].len();

        Self {
            tiles,
            width,
            height,
        }
    }
}
