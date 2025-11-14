use crate::ast::{BuildingDef, BuildingInstanceArray, City, PlayerType, PrereqArray, Production, ProductionType, UnitDef, UnitInstanceArray};
use super::map::GameMap;

#[derive(Debug)]
pub struct Civilization {
    pub resources: Resources,
    pub city: City,
}

#[derive(Debug)]
pub struct Resources {
    pub ressources: i32,
}

#[derive(Debug)]
pub struct GameState {
    pub map: GameMap,
    pub turn: i32,
    pub year: i32,

    // Civilizations
    pub civilizations: Vec<Civilization>,

    // Added: seed input and editing state
    pub seed_editing: bool,

    // Camera/viewport
    pub camera_x: i32,
    pub camera_y: i32,
    pub camera_mode: bool,

    // definition
    pub buildings: Vec<BuildingDef>,
    pub units: Vec<UnitDef>,

    // victory conditions
    pub nbTurns: u32,
    pub resourcesSpent: u32,

    pub zoom_level: u8, // 1, 2, or 3
}

impl GameState {
    pub fn new() -> Self {
        Self {
            map: GameMap::new_random(160usize, 40usize),
            turn: 1,
            year: -2500,

            civilizations: Vec::from([
                Civilization {
                    resources: Resources { ressources: 100 },
                    city: City {
                        name: "Player".to_string(),
                        x: 10,
                        y: 10,
                        buildings: BuildingInstanceArray { elements: Vec::new() },
                        blacklist_buildings: None,
                        blacklist_units: None,
                        color: "#0000FF".into(),
                        nbSlotsBuildings: 5,
                        nbSlotsUnits: 10,
                        playerType: PlayerType::PLAYER,
                        startingResources: 40,
                        units: UnitInstanceArray { units: Vec::new() },
                        whitelist_buildings: None,
                        whitelist_units: None,
                    },
                },
                Civilization {
                    resources: Resources { ressources: 100 },
                    city: City {
                        name: "IA".to_string(),
                        x: 20,
                        y: 20,
                        buildings: BuildingInstanceArray { elements: Vec::new() },
                        blacklist_buildings: None,
                        blacklist_units: None,
                        color: "#FF0000".into(),
                        nbSlotsBuildings: 5,
                        nbSlotsUnits: 10,
                        playerType: PlayerType::AI,
                        startingResources: 40,
                        units: UnitInstanceArray { units: Vec::new() },
                        whitelist_buildings: None,
                        whitelist_units: None,
                    },
                }
            ]),

            seed_editing: false,
            camera_x: 0,
            camera_y: 0,
            camera_mode: false,
            zoom_level: 1,
            buildings: Vec::from([
                BuildingDef {
                    name: "Farm".to_string(),
                    cost: 10,
                    buildTime: 2,
                    prerequisites: PrereqArray { prereqs: Vec::new() },
                    production: Production {
                        amount: 5,
                        cost: 0,
                        prodType: ProductionType::ressource,
                        prodUnitId: None,
                        time: 1,
                    },
                    slots: 1,
                },
                BuildingDef {
                    name: "Barracks".to_string(),
                    cost: 20,
                    buildTime: 4,
                    prerequisites: PrereqArray { prereqs: Vec::new() },
                    production: Production {
                        amount: 0,
                        cost: 5,
                        prodType: ProductionType::unit,
                        prodUnitId: Some("Warrior".to_string()),
                        time: 3,
                    },
                    slots: 1,
                },
            ]),
            units: Vec::from([
                UnitDef {
                    name: "Warrior".to_string(),
                    attack: 1,
                },
            ]),
            nbTurns: 500,
            resourcesSpent: 300,
        }
    }

    // Toggle editing state for the seed input
    pub fn toggle_seed_edit(&mut self) {
        self.seed_editing = !self.seed_editing;
    }

    // Add a character to the seed input (when editing)
    pub fn add_seed_char(&mut self, ch: char) {
        if self.seed_editing {
            self.map.seed.push(ch);
        }
    }

    // Remove last character from seed input
    pub fn backspace_seed(&mut self) {
        if self.seed_editing {
            self.map.seed.pop();
        }
    }

    // Apply the current seed: rebuild the map with the seed and stop editing
    pub fn submit_seed(&mut self) {
        self.map = GameMap::new(self.map.seed.clone(), self.map.width, self.map.height);
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
