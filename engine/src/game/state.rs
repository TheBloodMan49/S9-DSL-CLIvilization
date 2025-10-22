use super::map::{GameMap, Terrain};

pub struct Resources {
    pub gold: i32,
    pub science: i32,
    pub culture: i32,
}

pub struct City {
    pub population: i32,
}

pub struct GameState {
    pub map: GameMap,
    pub resources: Resources,
    pub cities: Vec<City>,
    pub turn: i32,
    pub year: i32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            map: GameMap::new("roblox".to_string()),
            resources: Resources {
                gold: 100,
                science: 0,
                culture: 0,
            },
            cities: vec![City { population: 1 }],
            turn: 1,
            year: -2500,
        }
    }
}
