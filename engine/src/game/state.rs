use crate::ast::{BuildingDef, BuildingInstance, BuildingInstanceArray, City, PlayerType, PrereqArray, Production, ProductionType, UnitDef, UnitInstance, UnitInstanceArray};
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

    pub player_turn: usize,

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
    pub nb_turns: u32,
    pub resources_spent: u32,

    pub zoom_level: u8, // 1, 2, or 3
    // Action input and popup
    pub action_editing: bool,
    pub action_input: String,
    pub popup: Option<Popup>,
}

#[derive(Debug, Clone)]
pub struct Popup {
    pub title: String,
    pub prompt: String,
    pub choices: Vec<String>,
    pub input: String,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            map: GameMap::new_random(160usize, 40usize),
            turn: 1,
            player_turn: 0,

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
                        nb_slots_buildings: 5,
                        nb_slots_units: 10,
                        player_type: PlayerType::PLAYER,
                        starting_resources: 40,
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
                        nb_slots_buildings: 5,
                        nb_slots_units: 10,
                        player_type: PlayerType::AI,
                        starting_resources: 40,
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
            action_editing: false,
            action_input: String::new(),
            popup: None,
            buildings: Vec::from([
                BuildingDef {
                    name: "Farm".to_string(),
                    cost: 10,
                    build_time: 2,
                    prerequisites: PrereqArray { prereqs: Vec::new() },
                    production: Production {
                        amount: 5,
                        cost: 0,
                        prod_type: ProductionType::RESSOURCE,
                        prod_unit_id: None,
                        time: 1,
                    },
                    slots: 1,
                },
                BuildingDef {
                    name: "Barracks".to_string(),
                    cost: 20,
                    build_time: 4,
                    prerequisites: PrereqArray { prereqs: Vec::new() },
                    production: Production {
                        amount: 0,
                        cost: 5,
                        prod_type: ProductionType::UNIT,
                        prod_unit_id: Some("Warrior".to_string()),
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
            nb_turns: 500,
            resources_spent: 300,
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

    // Action input helpers
    pub fn start_action_input(&mut self) {
        self.action_input.clear();
        self.action_editing = true;
    }

    pub fn add_action_char(&mut self, ch: char) {
        if self.action_editing {
            self.action_input.push(ch);
        }
    }

    pub fn backspace_action(&mut self) {
        if self.action_editing {
            self.action_input.pop();
        }
    }

    // Open a popup with choices and optional input
    pub fn open_popup(&mut self, title: &str, prompt: &str, choices: Vec<String>) {
        self.popup = Some(Popup {
            title: title.to_string(),
            prompt: prompt.to_string(),
            choices,
            input: String::new(),
        });
        // stop editing action while popup is open
        self.action_editing = false;
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    // submit the current action text; returns true if a popup was opened for further input
    pub fn submit_action(&mut self) -> bool {
        let txt = self.action_input.trim().to_lowercase();
        if txt.is_empty() {
            self.action_editing = false;
            self.action_input.clear();
            return false;
        }

        // end turn
        if txt == "end" || txt == "end turn" {
            self.player_turn = (self.player_turn + 1) % self.civilizations.len();
            if self.player_turn == 0 {
                self.turn += 1;
            }
            self.action_input.clear();
            self.action_editing = false;
            return false;
        }

        let parts: Vec<&str> = txt.split_whitespace().collect();
        match parts.get(0).map(|s| *s) {
            Some("build") => {
                // build [type]
                if parts.len() < 2 {
                    // open popup to choose building type
                    let choices = self.buildings.iter().map(|b| b.name.clone()).collect();
                    self.open_popup("Build", "Choose building type:", choices);
                    return true;
                } else {
                    let bname = parts[1];
                    if let Some(bdef) = self.buildings.iter().find(|b| b.name.to_lowercase() == bname) {
                        let civ = &mut self.civilizations[self.player_turn];
                        if civ.resources.ressources >= bdef.cost as i32 {
                            civ.resources.ressources -= bdef.cost as i32;
                            civ.city.buildings.elements.push(BuildingInstance { id_building: bdef.name.clone(), level: 1 });
                        } else {
                            self.open_popup("Build", "Not enough resources for building", vec![]);
                            return true;
                        }
                    } else {
                        self.open_popup("Build", &format!("Unknown building: {}", bname), vec![]);
                        return true;
                    }
                }
            }
            Some("hire") | Some("recruit") => {
                if parts.len() < 2 {
                    let choices = self.units.iter().map(|u| u.name.clone()).collect();
                    self.open_popup("Hire", "Choose unit to hire:", choices);
                    return true;
                } else {
                    let uname = parts[1];
                    if let Some(udef) = self.units.iter().find(|u| u.name.to_lowercase() == uname) {
                        let civ = &mut self.civilizations[self.player_turn];
                        // simple cost model: use building production.cost if any else 0; here we just push one unit
                        civ.city.units.units.push(UnitInstance { id_units: udef.name.clone(), nb_units: 1 });
                    } else {
                        self.open_popup("Hire", &format!("Unknown unit: {}", uname), vec![]);
                        return true;
                    }
                }
            }
            Some("attack") => {
                if parts.len() < 2 {
                    // choose target player
                    let choices = self.civilizations.iter().enumerate().filter(|(i,_)| *i != self.player_turn).map(|(_,c)| c.city.name.clone()).collect();
                    self.open_popup("Attack", "Choose player to attack:", choices);
                    return true;
                } else {
                    let target = parts[1];
                    if let Some((idx, _)) = self.civilizations.iter().enumerate().find(|(_,c)| c.city.name.to_lowercase() == target) {
                        let attacker_power = self.calculate_city_power(self.player_turn);
                        let defender_power = self.calculate_city_power(idx);
                        if attacker_power > defender_power {
                            // simple effect: transfer some resources
                            let stolen = 5;
                            let taken = stolen.min(self.civilizations[idx].resources.ressources);
                            self.civilizations[idx].resources.ressources -= taken;
                            self.civilizations[self.player_turn].resources.ressources += taken;
                        } else {
                            // failed attack: lose some resources
                            let loss = 3.min(self.civilizations[self.player_turn].resources.ressources);
                            self.civilizations[self.player_turn].resources.ressources -= loss;
                        }
                    } else {
                        self.open_popup("Attack", &format!("Unknown target: {}", target), vec![]);
                        return true;
                    }
                }
            }
            _ => {
                self.open_popup("Action", &format!("Unknown action: {}", txt), vec![]);
                return true;
            }
        }

        // default: clear action
        self.action_input.clear();
        self.action_editing = false;
        false
    }

    // submit popup input (interpret selection or text)
    pub fn submit_popup(&mut self) {
        if self.popup.is_none() { return; }
        let popup = self.popup.as_ref().unwrap().clone();
        // if choices exist, try to parse input as index or name
        if !popup.choices.is_empty() {
            let sel = popup.input.trim();
            let mut chosen: Option<String> = None;
            if let Ok(idx) = sel.parse::<usize>() {
                if idx >= 1 && idx <= popup.choices.len() {
                    chosen = Some(popup.choices[idx-1].clone());
                }
            }
            if chosen.is_none() {
                // try match by name
                for c in &popup.choices {
                    if c.to_lowercase().starts_with(&sel.to_lowercase()) {
                        chosen = Some(c.clone());
                        break;
                    }
                }
            }

            if let Some(ch) = chosen {
                // interpret by popup title
                match popup.title.as_str() {
                    "Build" => {
                        if let Some(bdef) = self.buildings.iter().find(|b| b.name == ch) {
                            let civ = &mut self.civilizations[self.player_turn];
                            if civ.resources.ressources >= bdef.cost as i32 {
                                civ.resources.ressources -= bdef.cost as i32;
                                civ.city.buildings.elements.push(BuildingInstance { id_building: bdef.name.clone(), level: 1 });
                            }
                        }
                    }
                    "Hire" => {
                        if let Some(udef) = self.units.iter().find(|u| u.name == ch) {
                            let civ = &mut self.civilizations[self.player_turn];
                            civ.city.units.units.push(UnitInstance { id_units: udef.name.clone(), nb_units: 1 });
                        }
                    }
                    "Attack" => {
                        if let Some((idx, _)) = self.civilizations.iter().enumerate().find(|(_,c)| c.city.name == ch) {
                            let attacker_power = self.calculate_city_power(self.player_turn);
                            let defender_power = self.calculate_city_power(idx);
                            if attacker_power > defender_power {
                                let stolen = 5;
                                let taken = stolen.min(self.civilizations[idx].resources.ressources);
                                self.civilizations[idx].resources.ressources -= taken;
                                self.civilizations[self.player_turn].resources.ressources += taken;
                            } else {
                                let loss = 3.min(self.civilizations[self.player_turn].resources.ressources);
                                self.civilizations[self.player_turn].resources.ressources -= loss;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // close popup after handling
        self.close_popup();
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

    pub fn calculate_city_power(&self, civ_index: usize) -> i32 {
        let civ = &self.civilizations[civ_index];
        let mut power = 0;

        // Power from units
        for unit in &civ.city.units.units {
            let id = &unit.id_units;
            power += unit.nb_units as i32 * self.units.iter()
                .find(|u| &u.name == id)
                .map_or(0, |u| u.attack as i32);
        }

        power
    }
}
