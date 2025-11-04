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

    // Camera/viewport
    pub camera_x: i32,
    pub camera_y: i32,
    pub camera_mode: bool,

    pub zoom_level: u8, // 1, 2, or 3
}

impl GameState {
    pub fn new(map: GameMap) -> Self {
        Self {
            map: map.clone(),
            resources: Resources {
                gold: 100,
                science: 0,
                culture: 0,
            },
            cities: vec![City { population: 1 }],
            turn: 1,
            year: -2500,
            seed_input: map.seed,
            seed_editing: false,
            camera_x: 0,
            camera_y: 0,
            camera_mode: false,
            zoom_level: 1,
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
        self.map = GameMap::new(self.seed_input.clone(), self.map.width, self.map.height);
        self.seed_editing = false;
    }

    pub fn toggle_camera_mode(&mut self) {
        self.camera_mode = !self.camera_mode;
    }

    pub fn move_camera(&mut self, dx: i32, dy: i32) {
        if self.camera_mode {
            self.camera_x = (self.camera_x + dx).clamp(0, self.map.width as i32 - 1);
            self.camera_y = (self.camera_y + dy).clamp(0, self.map.height as i32 - 1);
        }
    }

    pub fn cycle_zoom(&mut self) {
        let old_zoom = self.zoom_level;
        self.zoom_level = match self.zoom_level {
            1 => 2,
            2 => 3,
            _ => 1,
        };

        match (old_zoom, self.zoom_level) {
            (1, 2) => {
                // Zooming from 1x to 2x: multiply by 2
                self.camera_x *= 2;
                self.camera_y *= 2;
            }
            (2, 3) => {
                // Zooming from 2x to 3x: multiply by 3/2
                self.camera_x = (self.camera_x * 3) / 2;
                self.camera_y = (self.camera_y * 3) / 2;
            }
            (3, 1) => {
                // Zooming from 3x back to 1x: divide by 3
                self.camera_x /= 3;
                self.camera_y /= 3;
            }
            _ => {}
        }

        self.camera_x = self.camera_x.clamp(0, self.map.width as i32 - 1);
        self.camera_y = self.camera_y.clamp(0, self.map.height as i32 - 1);
    }
}
