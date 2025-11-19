use crate::ast::{BuildingDef, BuildingInstance, BuildingInstanceArray, City, PlayerType, PrereqArray, Production, ProductionType, UnitDef, UnitInstance, UnitInstanceArray};
use super::map::GameMap;

#[derive(Debug)]
pub struct Civilization {
    pub resources: Resources,
    pub city: City,
    // in-progress constructions and recruitments
    pub constructions: Vec<Construction>,
    pub recruitments: Vec<Recruitment>,
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

#[derive(Debug, Clone)]
pub struct Construction {
    pub id_building: String,
    pub remaining: u32,
    pub total: u32,
}

#[derive(Debug, Clone)]
pub struct Recruitment {
    pub id_unit: String,
    pub remaining: u32,
    pub amount: u32,
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
                    constructions: Vec::new(),
                    recruitments: Vec::new(),
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
                    constructions: Vec::new(),
                    recruitments: Vec::new(),
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
                                // attempt to start construction
                                let name = bdef.name.clone();
                                match self.start_construction(self.player_turn, &name) {
                                    Ok(()) => {}
                                    Err(err) => { self.open_popup("Build", &err, vec![]); return true; }
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
                                let uname_owned = udef.name.clone();
                                match self.start_recruitment(self.player_turn, &uname_owned) {
                                    Ok(()) => {}
                                    Err(err) => { self.open_popup("Hire", &err, vec![]); return true; }
                                }
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
                            let name = bdef.name.clone();
                            if let Err(err) = self.start_construction(self.player_turn, &name) {
                                self.open_popup("Build", &err, vec![]);
                                return;
                            }
                        }
                    }
                    "Hire" => {
                        if let Some(udef) = self.units.iter().find(|u| u.name == ch) {
                            let name = udef.name.clone();
                            if let Err(err) = self.start_recruitment(self.player_turn, &name) {
                                self.open_popup("Hire", &err, vec![]);
                                return;
                            }
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
        // reset action input
        self.action_input.clear();
        self.action_editing = false;
    }

    // Start construction: occupies a building slot immediately, finishes after build_time turns
    pub fn start_construction(&mut self, civ_index: usize, building_name: &str) -> Result<(), String> {
        let bdef = match self.buildings.iter().find(|b| b.name == building_name) {
            Some(b) => b,
            None => return Err(format!("Unknown building: {}", building_name)),
        };
        let civ = &mut self.civilizations[civ_index];
        let occupied = civ.city.buildings.elements.len() + civ.constructions.len();
        if occupied >= civ.city.nb_slots_buildings as usize {
            return Err("No available building slots".to_string());
        }
        if civ.resources.ressources < bdef.cost as i32 {
            return Err("Not enough resources for building".to_string());
        }
        civ.resources.ressources -= bdef.cost as i32;
        civ.constructions.push(Construction { id_building: bdef.name.clone(), remaining: bdef.build_time, total: bdef.build_time });
        Ok(())
    }

    // Start recruitment: requires an already-built building that produces this unit
    pub fn start_recruitment(&mut self, civ_index: usize, unit_name: &str) -> Result<(), String> {
        let udef = match self.units.iter().find(|u| u.name == unit_name) {
            Some(u) => u,
            None => return Err(format!("Unknown unit: {}", unit_name)),
        };
        let civ = &mut self.civilizations[civ_index];
        // check for building that can produce this unit (built only)
        let mut producer: Option<&BuildingDef> = None;
        for b_inst in &civ.city.buildings.elements {
            if let Some(bdef) = self.buildings.iter().find(|b| b.name == b_inst.id_building) {
                if format!("{:?}", bdef.production.prod_type).to_lowercase() == "unit" {
                    if let Some(prod_id) = &bdef.production.prod_unit_id {
                        if prod_id == &udef.name { producer = Some(bdef); break; }
                    }
                }
            }
        }
        if producer.is_none() {
            return Err("No building able to produce this unit is present".to_string());
        }
        let occupied_units = civ.city.units.units.len() + civ.recruitments.len();
        if occupied_units >= civ.city.nb_slots_units as usize {
            return Err("No available unit slots".to_string());
        }
        // use producer's production time and cost
        let bdef = producer.unwrap();
        let cost = bdef.production.cost as i32;
        if civ.resources.ressources < cost {
            return Err("Not enough resources to recruit unit".to_string());
        }
        civ.resources.ressources -= cost;
        civ.recruitments.push(Recruitment { id_unit: udef.name.clone(), remaining: bdef.production.time, amount: 1 });
        Ok(())
    }

    // Called at the start of each turn: decrease timers, finalize constructions/recruits, give resource production
    pub fn on_turn_start(&mut self) {
        // resources from resource-producing buildings
        for civ in &mut self.civilizations {
            // resource from finished buildings
            for b_inst in &civ.city.buildings.elements {
                if let Some(bdef) = self.buildings.iter().find(|b| b.name == b_inst.id_building) {
                    if format!("{:?}", bdef.production.prod_type).to_lowercase() == "ressource" {
                        civ.resources.ressources += bdef.production.amount as i32;
                    }
                }
            }

            // process constructions
            let mut finished_builds: Vec<usize> = Vec::new();
            for (i, cons) in civ.constructions.iter_mut().enumerate() {
                if cons.remaining > 0 { cons.remaining -= 1; }
                if cons.remaining == 0 { finished_builds.push(i); }
            }
            // finalize in reverse order to remove by index safely
            for idx in finished_builds.into_iter().rev() {
                let cons = civ.constructions.remove(idx);
                civ.city.buildings.elements.push(BuildingInstance { id_building: cons.id_building, level: 1 });
            }

            // process recruitments
            let mut finished_recruits: Vec<usize> = Vec::new();
            for (i, rec) in civ.recruitments.iter_mut().enumerate() {
                if rec.remaining > 0 { rec.remaining -= 1; }
                if rec.remaining == 0 { finished_recruits.push(i); }
            }
            for idx in finished_recruits.into_iter().rev() {
                let rec = civ.recruitments.remove(idx);
                // add unit instance (merge if existing)
                if let Some(ui) = civ.city.units.units.iter_mut().find(|u| u.id_units == rec.id_unit) {
                    ui.nb_units += rec.amount;
                } else {
                    civ.city.units.units.push(UnitInstance { id_units: rec.id_unit, nb_units: rec.amount });
                }
            }
        }
        // increment turn counter maybe handled elsewhere; keep turn as-is here
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
