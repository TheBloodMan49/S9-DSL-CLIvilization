use super::map::GameMap;

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

    // Added: seed input and editing state
    pub seed_input: String,
    pub seed_editing: bool,
}

impl GameState {
    pub fn new() -> Self {
        let initial_seed = "pokemon".to_string();
        Self {
            map: GameMap::new(initial_seed.clone()),
            resources: Resources {
                gold: 100,
                science: 0,
                culture: 0,
            },
            cities: vec![City { population: 1 }],
            turn: 1,
            year: -2500,
            seed_input: initial_seed,
            seed_editing: false,
        }
    }

    // Toggle editing state for the seed input
    pub fn toggle_seed_edit(&mut self) {
        self.seed_editing = !self.seed_editing;
    }

    // Add a character to the seed input (when editing)
    pub fn add_seed_char(&mut self, ch: char) {
        if self.seed_editing {
            self.seed_input.push(ch);
        }
    }

    // Remove last character from seed input
    pub fn backspace_seed(&mut self) {
        if self.seed_editing {
            self.seed_input.pop();
        }
    }

    // Apply the current seed: rebuild the map with the seed and stop editing
    pub fn submit_seed(&mut self) {
        self.map = GameMap::new(self.seed_input.clone());
        self.seed_editing = false;
    }
}
