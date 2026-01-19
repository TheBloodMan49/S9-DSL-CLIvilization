use super::map::{GameMap, Terrain};
use std::collections::{BinaryHeap};
use std::cmp::Reverse;
use crate::ast::{
    BuildingDef, BuildingInstance, BuildingInstanceArray, City, PlayerType, PrereqArray,
    Production, ProductionType, UnitDef, UnitInstance, UnitInstanceArray,
};
use ratatui::style::Color;
use log::{debug, info, warn};
use anyhow::{Result, anyhow};

/// Represents a civilization (player) in the game.
///
/// Each civilization has resources, a city with buildings and units,
/// and can be alive or defeated.
#[derive(Debug)]
pub struct Civilization {
    /// Resource pool
    pub resources: Resources,
    /// The civilization's main city
    pub city: City,
    /// Whether the civilization is still alive
    pub alive: bool,
    /// In-progress constructions (buildings being built)
    pub constructions: Vec<Construction>,
    /// In-progress recruitments (units being trained)
    pub recruitments: Vec<Recruitment>,
}

/// Resource pool for a civilization.
#[derive(Debug)]
pub struct Resources {
    /// Amount of resources available. The game uses a single resource type currently.
    pub ressources: i32,
}

/// Core game state aggregating map, players, turns, and UI state. Mutable caching fields optimize rendering hot paths.
#[derive(Debug)]
pub struct GameState {
    pub map: GameMap,
    pub turn: i32,

    pub player_turn: usize,

    /// All civilizations in the game
    pub civilizations: Vec<Civilization>,

    /// Whether the seed input field is being edited
    pub seed_editing: bool,

    /// Camera position for viewing the map
    pub camera_x: i32,
    pub camera_y: i32,
    /// Whether camera mode is active (for panning)
    pub camera_mode: bool,
    /// Cached map rendering buffer
    pub map_buffer_cache: Option<Vec<Vec<Color>>>,

    /// Building and unit definitions (templates)
    pub buildings: Vec<BuildingDef>,
    pub units: Vec<UnitDef>,

    /// Victory conditions
    pub nb_turns: u32,
    pub resources_spent: u32,

    /// Zoom level for map rendering (1, 2, or 3)
    pub zoom_level: u8,

    /// Action input state
    pub action_editing: bool,
    pub action_input: String,

    /// Currently open popup (if any)
    pub popup: Option<Popup>,

    /// Active travels (attacks in transit)
    pub travels: Vec<Travel>,

    /// Whether the game is over
    pub game_over: bool,

    /// Whether an AI is currently thinking/acting
    pub ai_thinking: bool,
}

/// A popup dialog shown to the user for choices or information.
#[derive(Debug, Clone)]
pub struct Popup {
    pub title: String,
    pub prompt: String,
    /// Available choices (if any)
    pub choices: Vec<String>,
    /// User's input/selection
    pub input: String,
}

/// An in-progress building construction.
#[derive(Debug, Clone)]
pub struct Construction {
    pub id_building: String,
    pub remaining: u32,
    pub total: u32,
}

/// An in-progress unit recruitment.
#[derive(Debug, Clone)]
pub struct Recruitment {
    pub id_unit: String,
    pub remaining: u32,
    pub amount: u32,
}

/// A traveling attack force.
///
/// Represents units moving from one city to attack another.
/// The attack resolves when remaining reaches 0.
#[derive(Debug, Clone)]
pub struct Travel {
    pub attacker: usize,
    pub defender: usize,
    pub amount: u32,
    pub remaining: u32,
    pub total: u32,
    pub path: Vec<(i32, i32)>,
}

impl GameState {
    /// Create default game with two civilizations and procedural map. Provides playable starting state without config.
    ///
    /// Initializes a game with:
    /// - A random map
    /// - Two civilizations (one player, one AI)
    /// - Default buildings and units
    /// - Starting resources
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
                        buildings: BuildingInstanceArray {
                            elements: Vec::new(),
                        },
                        blacklist_buildings: None,
                        blacklist_units: None,
                        color: "#8325D5".into(),
                        nb_slots_buildings: 5,
                        nb_slots_units: 10,
                        player_type: PlayerType::PLAYER,
                        starting_resources: 40,
                        units: UnitInstanceArray { units: Vec::new() },
                        whitelist_buildings: None,
                        whitelist_units: None,
                    },
                    alive: true,
                    constructions: Vec::new(),
                    recruitments: Vec::new(),
                },
                Civilization {
                    resources: Resources { ressources: 100 },
                    city: City {
                        name: "IA".to_string(),
                        x: 20,
                        y: 20,
                        buildings: BuildingInstanceArray {
                            elements: Vec::new(),
                        },
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
                    alive: true,
                    constructions: Vec::new(),
                    recruitments: Vec::new(),
                },
            ]),

            seed_editing: false,
            camera_x: 0,
            camera_y: 0,
            camera_mode: false,
            map_buffer_cache: None,
            zoom_level: 1,
            action_editing: false,
            action_input: String::new(),
            popup: None,
            travels: Vec::new(),
            game_over: false,
            ai_thinking: false,
            buildings: Vec::from([
                BuildingDef {
                    name: "Farm".to_string(),
                    cost: 10,
                    build_time: 2,
                    prerequisites: PrereqArray {
                        prereqs: Vec::new(),
                    },
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
                    prerequisites: PrereqArray {
                        prereqs: Vec::new(),
                    },
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
            units: Vec::from([UnitDef {
                name: "Warrior".to_string(),
                attack: 1,
            }]),
            nb_turns: 500,
            resources_spent: 300,
        }
    }

    /// Toggle seed editing mode on/off.
    pub fn toggle_seed_edit(&mut self) {
        self.seed_editing = !self.seed_editing;
    }

    /// Add a character to the seed input while editing.
    ///
    /// # Arguments
    /// * `ch` - Character to add
    pub fn add_seed_char(&mut self, ch: char) {
        if self.seed_editing {
            self.map.seed.push(ch);
        }
    }

    /// Remove the last character from the seed input.
    pub fn backspace_seed(&mut self) {
        if self.seed_editing {
            self.map.seed.pop();
        }
    }

    /// Regenerate map from current seed and exit edit mode. Atomic operation ensures consistent state.
    pub fn submit_seed(&mut self) {
        self.map = GameMap::new(self.map.seed.clone(), self.map.width, self.map.height);
        self.seed_editing = false;
    }

    /// Toggle camera mode on/off.
    pub fn toggle_camera_mode(&mut self) {
        self.camera_mode = !self.camera_mode;
    }

    /// Start editing an action input.
    pub fn start_action_input(&mut self) {
        self.action_input.clear();
        self.action_editing = true;
    }

    /// Add a character to the action input while editing.
    ///
    /// # Arguments
    /// * `ch` - Character to add
    pub fn add_action_char(&mut self, ch: char) {
        if self.action_editing {
            self.action_input.push(ch);
        }
    }

    /// Remove the last character from the action input.
    pub fn backspace_action(&mut self) {
        if self.action_editing {
            self.action_input.pop();
        }
    }

    /// Open a popup dialog with choices and optional input.
    ///
    /// # Arguments
    /// * `title` - Popup title
    /// * `prompt` - Prompt text
    /// * `choices` - List of available choices (empty if free text input)
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

    /// Close the currently open popup.
    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    /// Parse and execute action with automatic popup generation for missing parameters. Lowercase parsing provides case-insensitive UX.
    ///
    /// # Returns
    /// true if a popup was opened for further input, false otherwise
    pub fn submit_action(&mut self) -> bool {
        let txt = self.action_input.trim().to_lowercase();
        debug!("submit_action called (player={}): '{}'", self.player_turn, txt);
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
            // process turn start effects
            info!("Player ended turn; new player_turn={} turn={}", self.player_turn, self.turn);
            self.on_turn_start(self.player_turn);

            self.action_input.clear();
            self.action_editing = false;
            return false;
        }

        let parts: Vec<&str> = txt.split_whitespace().collect();
        match parts.first().copied() {
            Some("build") => {
                // build [type]
                if parts.len() < 2 {
                    // open popup to choose building type
                    let choices = self.buildings.iter().map(|b| b.name.clone()).collect();
                    debug!("Opening Build popup for player {} (no building specified)", self.player_turn);
                    self.open_popup("Build", "Choose building type:", choices);
                    return true;
                } else {
                    let bname = parts[1];
                    if let Some(bdef) = self
                        .buildings
                        .iter()
                        .find(|b| b.name.to_lowercase() == bname)
                    {
                        // attempt to start construction
                        let name = bdef.name.clone();
                        match self.start_construction(self.player_turn, &name) {
                            Ok(()) => {
                                info!("Started construction '{}' for civ {}", name, self.player_turn);
                            }
                            Err(err) => {
                                warn!("Failed to start construction for civ {}: {}", self.player_turn, err);
                                self.open_popup("Build", &format!("{:#}", err), vec![]);
                                return true;
                            }
                        }
                    } else {
                        warn!("Unknown building requested by player {}: {}", self.player_turn, bname);
                        self.open_popup("Build", &format!("Unknown building: {bname}"), vec![]);
                        return true;
                    }
                }
            }
            Some("hire" | "recruit") => {
                if parts.len() < 2 {
                    let choices = self.units.iter().map(|u| u.name.clone()).collect();
                    debug!("Opening Hire popup for player {} (no unit specified)", self.player_turn);
                    self.open_popup("Hire", "Choose unit to hire:", choices);
                    return true;
                } else {
                    let uname = parts[1];
                    if let Some(udef) = self.units.iter().find(|u| u.name.to_lowercase() == uname) {
                        let uname_owned = udef.name.clone();
                        match self.start_recruitment(self.player_turn, &uname_owned) {
                            Ok(()) => {
                                info!("Started recruitment '{}' for civ {}", uname_owned, self.player_turn);
                            }
                            Err(err) => {
                                warn!("Failed to start recruitment for civ {}: {}", self.player_turn, err);
                                self.open_popup("Hire", &format!("{:#}", err), vec![]);
                                return true;
                            }
                        }
                    } else {
                        warn!("Unknown unit requested by player {}: {}", self.player_turn, uname);
                        self.open_popup("Hire", &format!("Unknown unit: {uname}"), vec![]);
                        return true;
                    }
                }
            }
            Some("attack") => {
                if parts.len() < 2 {
                    // choose target player
                    let choices = self
                        .civilizations
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != self.player_turn)
                        .map(|(_, c)| c.city.name.clone())
                        .collect();
                    debug!("Opening Attack popup for player {} (no target specified)", self.player_turn);
                    self.open_popup("Attack", "Choose player to attack:", choices);
                    return true;
                } else {
                    let target = parts[1];
                    if let Some((idx, _)) = self
                        .civilizations
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.city.name.to_lowercase() == target)
                    {
                        // optional amount as third argument
                        let amount = if parts.len() >= 3 {
                            parts[2].parse::<u32>().ok()
                        } else {
                            None
                        };
                        match self.start_attack(self.player_turn, idx, amount) {
                            Ok(()) => {
                                info!("Started attack from {} to {} (amount {:?})", self.player_turn, idx, amount);
                            }
                            Err(e) => {
                                warn!("Failed to start attack for civ {}: {}", self.player_turn, e);
                                self.open_popup("Attack", &format!("{:#}", e), vec![]);
                                return true;
                            }
                        }
                    } else {
                        warn!("Unknown attack target requested by player {}: {}", self.player_turn, target);
                        self.open_popup("Attack", &format!("Unknown target: {target}"), vec![]);
                        return true;
                    }
                }
            }
            _ => {
                warn!("Unknown action by player {}: {}", self.player_turn, txt);
                self.open_popup("Action", &format!("Unknown action: {txt}"), vec![]);
                return true;
            }
        }

        // default: clear action
        self.action_input.clear();
        self.action_editing = false;
        false
    }

    /// Handle popup submission with fuzzy choice matching. Accepts both numeric indices and name prefixes for flexibility.
    ///
    /// Interprets the user's selection (by index or name) and executes
    /// the corresponding action (build, hire, attack, etc.).
    pub fn submit_popup(&mut self) {
        if self.popup.is_none() {
            debug!("submit_popup called but no popup present (player {})", self.player_turn);
            return;
        }
        let popup = self.popup.as_ref().unwrap().clone();
        debug!("submit_popup called for '{}' (player {}), input='{}'", popup.title, self.player_turn, popup.input);
        // if choices exist, try to parse input as index or name
        if !popup.choices.is_empty() {
            let sel = popup.input.trim();
            let mut chosen: Option<String> = None;
            if let Ok(idx) = sel.parse::<usize>()
                && idx >= 1 && idx <= popup.choices.len() {
                    chosen = Some(popup.choices[idx - 1].clone());
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
                                warn!("start_construction failed in popup for civ {}: {}", self.player_turn, err);
                                self.open_popup("Build", &format!("{:#}", err), vec![]);
                                return;
                            }
                            info!("Construction started from popup for civ {}: {}", self.player_turn, name);
                        }
                    }
                    "Hire" => {
                        if let Some(udef) = self.units.iter().find(|u| u.name == ch) {
                            let name = udef.name.clone();
                            if let Err(err) = self.start_recruitment(self.player_turn, &name) {
                                warn!("start_recruitment failed in popup for civ {}: {}", self.player_turn, err);
                                self.open_popup("Hire", &format!("{:#}", err), vec![]);
                                return;
                            }
                            info!("Recruitment started from popup for civ {}: {}", self.player_turn, name);
                        }
                    }
                    "Attack" => {
                        if let Some((idx, _)) = self
                            .civilizations
                            .iter()
                            .enumerate()
                            .find(|(_, c)| c.city.name == ch)
                        {
                            if let Err(e) = self.start_attack(self.player_turn, idx, None) {
                                warn!("start_attack failed in popup for civ {}: {}", self.player_turn, e);
                                self.open_popup("Attack", &format!("{:#}", e), vec![]);
                                return;
                            }
                            info!("Attack started from popup for civ {} -> {}", self.player_turn, idx);
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

    /// Start a building construction for a civilization.
    ///
    /// This method:
    /// - Validates the building exists and can be built
    /// - Checks for available building slots
    /// - Deducts resources
    /// - Adds the construction to the in-progress queue
    ///
    /// # Arguments
    /// * `civ_index` - Index of the civilization building
    /// * `building_name` - Name of the building to construct
    ///
    /// # Returns
    /// Ok(()) on success, or an error describing why construction cannot start
    pub fn start_construction(
        &mut self,
        civ_index: usize,
        building_name: &str,
    ) -> Result<()> {
        debug!("start_construction called: civ={civ_index} building='{building_name}'");
        let bdef = if let Some(b) = self.buildings.iter().find(|b| b.name == building_name) { b } else {
            warn!("start_construction: unknown building '{building_name}' for civ {civ_index}");
            return Err(anyhow!("Unknown building: {building_name}"));
        };
        let civ = &mut self.civilizations[civ_index];
        let occupied = civ.city.buildings.elements.len() + civ.constructions.len();
        // Only one construction at a time
        if !civ.constructions.is_empty() {
            warn!("start_construction: another construction already in progress for civ {civ_index}");
            return Err(anyhow!("Another construction is already in progress"));
        }

        // check for available slots
        if occupied >= civ.city.nb_slots_buildings as usize {
            warn!("start_construction: no available building slots for civ {civ_index}");
            return Err(anyhow!("No available building slots"));
        }

        // check resources
        if civ.resources.ressources < bdef.cost as i32 {
            warn!("start_construction: not enough resources for civ {} (cost={})", civ_index, bdef.cost);
            return Err(anyhow!("Not enough resources for building"));
        }
        civ.resources.ressources -= bdef.cost as i32;
        civ.constructions.push(Construction {
            id_building: bdef.name.clone(),
            remaining: bdef.build_time,
            total: bdef.build_time,
        });
        Ok(())
    }

    /// Start unit recruitment for a civilization.
    ///
    /// This method:
    /// - Validates the unit exists and can be recruited
    /// - Checks for a building that can produce this unit
    /// - Checks for available unit slots
    /// - Deducts resources
    /// - Adds the recruitment to the in-progress queue
    ///
    /// # Arguments
    /// * `civ_index` - Index of the civilization recruiting
    /// * `unit_name` - Name of the unit to recruit
    ///
    /// # Returns
    /// Ok(()) on success, or an error describing why recruitment cannot start
    pub fn start_recruitment(&mut self, civ_index: usize, unit_name: &str) -> Result<()> {
        debug!("start_recruitment called: civ={civ_index} unit='{unit_name}'");
        let udef = if let Some(u) = self.units.iter().find(|u| u.name == unit_name) { u } else {
            warn!("start_recruitment: unknown unit '{unit_name}' for civ {civ_index}");
            return Err(anyhow!("Unknown unit: {unit_name}"));
        };
        let civ = &mut self.civilizations[civ_index];
        // check for building that can produce this unit (built only)
        let mut producer: Option<&BuildingDef> = None;
        for b_inst in &civ.city.buildings.elements {
            if let Some(bdef) = self.buildings.iter().find(|b| b.name == b_inst.id_building)
                && format!("{:?}", bdef.production.prod_type).to_lowercase() == "unit"
                    && let Some(prod_id) = &bdef.production.prod_unit_id
                        && prod_id == &udef.name {
                            producer = Some(bdef);
                            break;
                        }
        }
        // no producer found
        if producer.is_none() {
            warn!("start_recruitment: no producer building for unit '{unit_name}' civ {civ_index}");
            return Err(anyhow!("No building able to produce this unit is present"));
        }

        // only one recruitment at a time
        if !civ.recruitments.is_empty() {
            warn!("start_recruitment: recruitment already in progress for civ {civ_index}");
            return Err(anyhow!("Another recruitment is already in progress"));
        }

        // check for available unit slots
        let occupied_units = civ.city.units.units.len() + civ.recruitments.len();
        if occupied_units >= civ.city.nb_slots_units as usize {
            warn!("start_recruitment: no available unit slots for civ {civ_index}");
            return Err(anyhow!("No available unit slots"));
        }

        // use producer's production time and cost
        let bdef = producer.unwrap();
        let cost = bdef.production.cost as i32;
        if civ.resources.ressources < cost {
            warn!("start_recruitment: not enough resources for civ {civ_index} (cost={cost})");
            return Err(anyhow!("Not enough resources to recruit unit"));
        }
        civ.resources.ressources -= cost;
        civ.recruitments.push(Recruitment {
            id_unit: udef.name.clone(),
            remaining: bdef.production.time,
            amount: 1,
        });
        Ok(())
    }

    /// Called at the start of each turn for a player.
    ///
    /// This method:
    /// - Grants resources from buildings
    /// - Decrements construction and recruitment timers
    /// - Finalizes completed constructions and recruitments
    /// - Processes traveling attacks
    /// - Resolves battles
    /// - Checks for victory conditions
    ///
    /// # Arguments
    /// * `player_index` - Index of the player whose turn is starting
    pub fn on_turn_start(&mut self, player_index: usize) {
        info!("on_turn_start: player {} turn={}", player_index, self.turn);
        let civ = &mut self.civilizations[player_index];
        // resource from finished buildings
        for b_inst in &civ.city.buildings.elements {
            if let Some(bdef) = self.buildings.iter().find(|b| b.name == b_inst.id_building)
                && format!("{:?}", bdef.production.prod_type).to_lowercase() == "ressource" {
                    civ.resources.ressources += bdef.production.amount as i32;
                }
        }

        // process constructions
        let mut finished_builds: Vec<usize> = Vec::new();
        for (i, cons) in civ.constructions.iter_mut().enumerate() {
            if cons.remaining > 0 {
                cons.remaining -= 1;
            }
            if cons.remaining == 0 {
                finished_builds.push(i);
            }
        }
        // finalize in reverse order to remove by index safely
        for idx in finished_builds.into_iter().rev() {
            let cons = civ.constructions.remove(idx);
            let id = cons.id_building.clone();
            civ.city.buildings.elements.push(BuildingInstance { id_building: id.clone(), level: 1 });
            info!("Construction finished for civ {player_index}: {id}");
        }

        // process recruitments
        let mut finished_recruits: Vec<usize> = Vec::new();
        for (i, rec) in civ.recruitments.iter_mut().enumerate() {
            if rec.remaining > 0 {
                rec.remaining -= 1;
            }
            if rec.remaining == 0 {
                finished_recruits.push(i);
            }
        }
        for idx in finished_recruits.into_iter().rev() {
            let rec = civ.recruitments.remove(idx);
            // add unit instance (merge if existing)
            let id_unit = rec.id_unit.clone();
            if let Some(ui) = civ
                .city
                .units
                .units
                .iter_mut()
                .find(|u| u.id_units == id_unit)
            {
                ui.nb_units += rec.amount;
            } else {
                civ.city.units.units.push(UnitInstance { id_units: id_unit.clone(), nb_units: rec.amount });
            }
            info!("Recruitment finished for civ {}: {} (+{} units)", player_index, id_unit, rec.amount);
        }

        // process travels (attacks in transit)
        let mut arrived: Vec<usize> = Vec::new();
        for (i, t) in self.travels.iter_mut().enumerate() {
            if t.remaining > 0 {
                t.remaining -= 1;
            }
            if t.remaining == 0 {
                arrived.push(i);
            }
        }
        for idx in arrived.into_iter().rev() {
            let t = self.travels.remove(idx);
            // if either side is already dead, ignore
            if !self.civilizations[t.attacker].alive || !self.civilizations[t.defender].alive {
                continue;
            }

            let attacker_power = t.amount as i32;
            let defender_power = self.calculate_city_power(t.defender);

            if attacker_power > defender_power {
                // attacker wins: defender loses the game
                self.civilizations[t.defender].alive = false;
                // remove all defender units
                self.civilizations[t.defender].city.units.units.clear();
                // feedback popup
                self.open_popup(
                    "Battle",
                    &format!(
                        "{} attacked {} ({} vs {}) — defender eliminated",
                        self.civilizations[t.attacker].city.name,
                        self.civilizations[t.defender].city.name,
                        attacker_power,
                        defender_power
                    ),
                    vec![],
                );
                info!("Battle resolved: attacker {} defeated defender {}", t.attacker, t.defender);
            } else {
                // defender holds: attacker units are lost (they were removed when sent); defender loses some units as casualties
                let casualties = (attacker_power as u32) / 2;
                let lost = self.remove_units_from_city(t.defender, casualties);
                self.open_popup(
                    "Battle",
                    &format!(
                        "{} attacked {} ({} vs {}) — attack failed, defender lost {} units",
                        self.civilizations[t.attacker].city.name,
                        self.civilizations[t.defender].city.name,
                        attacker_power,
                        defender_power,
                        lost
                    ),
                    vec![],
                );
                info!("Battle resolved: attacker {} failed against {} (defender lost {} units)", t.attacker, t.defender, lost);
            }
        }

        // check victory: if only one alive remains, end game
        let alive_count = self.civilizations.iter().filter(|c| c.alive).count();
        if alive_count <= 1 && !self.game_over {
            self.game_over = true;
            if let Some(winner) = self.civilizations.iter().find(|c| c.alive) {
                self.open_popup(
                    "Game Over",
                    &format!("Winner: {}", winner.city.name),
                    vec![],
                );
            } else {
                self.open_popup("Game Over", "No winners", vec![]);
            }
        }
        // increment turn counter maybe handled elsewhere; keep turn as-is here
    }

    /// Remove units with smallest-first priority. Returns actual removed count for battle casualty reporting.
    ///
    /// # Arguments
    /// * `civ_index` - Index of the civilization
    /// * `to_remove` - Number of units to remove
    ///
    /// # Returns
    /// The actual number of units removed
    fn remove_units_from_city(&mut self, civ_index: usize, mut to_remove: u32) -> u32 {
        debug!("remove_units_from_city called: civ={civ_index} to_remove={to_remove}");
        let civ = &mut self.civilizations[civ_index];
        let mut removed: u32 = 0;
        let mut i = 0;
        while i < civ.city.units.units.len() && to_remove > 0 {
            let available: u32 = civ.city.units.units[i].nb_units;
            if available <= to_remove {
                removed += available;
                to_remove -= available;
                civ.city.units.units.remove(i);
                // do not increment i since we removed current
            } else {
                civ.city.units.units[i].nb_units = available - to_remove;
                removed += to_remove;
                to_remove = 0;
                i += 1;
            }
        }
        debug!("remove_units_from_city result: removed={removed} remaining_to_remove={to_remove}");
        removed
    }

    /// Launch attack with pathfinding and travel time calculation. Units removed immediately; combat deferred until arrival.
    /// Weighted pathfinding accounts for terrain: water slower than land, mountains impassable.
    ///
    /// # Arguments
    /// * `attacker_idx` - Index of the attacking civilization
    /// * `defender_idx` - Index of the defending civilization
    /// * `amount_opt` - Optional number of units to send (None = all units)
    ///
    /// # Returns
    /// Ok(()) on success, or an error describing why the attack cannot start
    pub fn start_attack(
        &mut self,
        attacker_idx: usize,
        defender_idx: usize,
        amount_opt: Option<u32>,
    ) -> Result<()> {
        if attacker_idx >= self.civilizations.len() || defender_idx >= self.civilizations.len() {
            return Err(anyhow!("Invalid civilization index"));
        }
        if attacker_idx == defender_idx {
            return Err(anyhow!("Cannot attack yourself"));
        }
        if self.game_over {
            return Err(anyhow!("Game is over"));
        }

        if !self.civilizations[attacker_idx].alive {
            return Err(anyhow!("Attacker is not alive"));
        }
        if !self.civilizations[defender_idx].alive {
            return Err(anyhow!("Target is already defeated"));
        }

        // count available units
        let total_units: u32 = self.civilizations[attacker_idx]
            .city
            .units
            .units
            .iter()
            .map(|u| u.nb_units)
            .sum();
        if total_units == 0 {
            return Err(anyhow!("No units available to send"));
        }

        let send_amount = amount_opt.unwrap_or(total_units).min(total_units);
        if send_amount == 0 {
            return Err(anyhow!("Invalid amount to send"));
        }

        // remove units from attacker immediately (they are now in transit)
        let removed = self.remove_units_from_city(attacker_idx, send_amount);
        if removed == 0 {
            return Err(anyhow!("Failed to remove units"));
        }

        // compute travel path using weighted shortest path allowing water (but not mountain)
        let a = &self.civilizations[attacker_idx].city;
        let b = &self.civilizations[defender_idx].city;
        let src = (a.x.cast_signed(), a.y.cast_signed());
        let dst = (b.x.cast_signed(), b.y.cast_signed());
        let path_opt = self.bfs_path(src, dst);
        if path_opt.is_none() {
            return Err(anyhow!("No path to target (blocked by terrain)"));
        }
        let path = path_opt.unwrap();
        // compute time to traverse the path accounting for water slowdown
        // default: land tiles move at 3 blocks/turn, water at 1 block/turn
        let land_speed = 3.0_f64; // blocks per turn on land
        let water_speed = 1.0_f64; // blocks per turn on water
        let mut total_time: f64 = 0.0;
        if path.len() >= 2 {
            for i in 1..path.len() {
                let (nx, ny) = path[i];
                let terrain = &self.map.tiles[ny as usize][nx as usize];
                let step_time = match terrain {
                    Terrain::Water => 1.0 / water_speed,
                    Terrain::Mountain => continue, // should not happen, mountain is impassable
                    _ => 1.0 / land_speed,
                };
                total_time += step_time;
            }
        }
        let mut turns = total_time.ceil() as u32;
        if turns == 0 { turns = 1; }

        self.travels.push(Travel {
            attacker: attacker_idx,
            defender: defender_idx,
            amount: removed,
            remaining: turns,
            total: turns,
            path,
        });
        Ok(())
    }

    /// Find a weighted shortest path from source to destination on the map.
    ///
    /// Uses Dijkstra's algorithm to find a path that:
    /// - Avoids mountains (impassable)
    /// - Allows water but with higher cost (slower movement)
    /// - Prefers land tiles
    ///
    /// # Arguments
    /// * `src` - Source coordinates (x, y)
    /// * `dst` - Destination coordinates (x, y)
    ///
    /// # Returns
    /// Some(path) if a path exists, None otherwise
    fn bfs_path(&self, src: (i32, i32), dst: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let width = self.map.width as i32;
        let height = self.map.height as i32;
        let (sx, sy) = src;
        let (dx, dy) = dst;
        if sx < 0 || sy < 0 || dx < 0 || dy < 0 { return None; }
        if sx >= width || sy >= height || dx >= width || dy >= height { return None; }

        // Dijkstra structures (integer scaled costs to avoid f64 ordering issues)
        const SCALE: i64 = 1000; // 1.0 turn == 1000 units
        let mut dist: Vec<Vec<i64>> = vec![vec![i64::MAX; width as usize]; height as usize];
        let mut parent: Vec<Vec<Option<(i32,i32)>>> = vec![vec![None; width as usize]; height as usize];
        // min-heap of (cost, x, y) using Reverse to get smallest cost
        let mut heap: BinaryHeap<Reverse<(i64, i32, i32)>> = BinaryHeap::new();

        dist[sy as usize][sx as usize] = 0;
        heap.push(Reverse((0, sx, sy)));

        let neighbors = [(-1,0),(1,0),(0,-1),(0,1)];
        while let Some(Reverse((cost, cx, cy))) = heap.pop() {
            if cost > dist[cy as usize][cx as usize] { continue; }
            if (cx, cy) == (dx, dy) {
                // reconstruct path
                let mut path = Vec::new();
                let mut cur = Some((cx, cy));
                while let Some(p) = cur {
                    path.push(p);
                    cur = parent[p.1 as usize][p.0 as usize];
                }
                path.reverse();
                return Some(path);
            }

            for (ox, oy) in neighbors.iter() {
                let nx = cx + ox;
                let ny = cy + oy;
                if nx < 0 || ny < 0 || nx >= width || ny >= height { continue; }
                // check terrain of destination tile
                let terrain = &self.map.tiles[ny as usize][nx as usize];
                if matches!(terrain, Terrain::Mountain) { continue; }
                // cost per step scaled as integers: water slower, land faster
                let step_cost_scaled: i64 = match terrain {
                    Terrain::Water => SCALE,        // 1.0 turn == SCALE units
                    _ => (SCALE as f64 / 3.0).round() as i64, // land: ~1/3 turn
                };
                let new_cost = cost.saturating_add(step_cost_scaled);
                if new_cost < dist[ny as usize][nx as usize] {
                    dist[ny as usize][nx as usize] = new_cost;
                    parent[ny as usize][nx as usize] = Some((cx, cy));
                    heap.push(Reverse((new_cost, nx, ny)));
                }
            }
        }

        None
    }

    /// Move the camera by the specified offset.
    ///
    /// Only works when camera mode is active.
    ///
    /// # Arguments
    /// * `dx` - Horizontal offset
    /// * `dy` - Vertical offset
    pub fn move_camera(&mut self, dx: i32, dy: i32) {
        if self.camera_mode {
            self.camera_x += dx;
            self.camera_y += dy;
        }
    }

    /// Cycle through zoom levels (1 -> 2 -> 3 -> 1).
    pub fn cycle_zoom(&mut self) {
        self.zoom_level = match self.zoom_level {
            1 => 2,
            2 => 3,
            _ => 1,
        };
    }

    /// Calculate the total military power of a civilization.
    ///
    /// Power is computed from units, weighted by their attack values.
    ///
    /// # Arguments
    /// * `civ_index` - Index of the civilization
    ///
    /// # Returns
    /// Total military power
    pub fn calculate_city_power(&self, civ_index: usize) -> i32 {
        let civ = &self.civilizations[civ_index];
        let mut power = 0;

        // Power from units
        for unit in &civ.city.units.units {
            let id = &unit.id_units;
            power += unit.nb_units as i32
                * self
                    .units
                    .iter()
                    .find(|u| &u.name == id)
                    .map_or(0, |u| u.attack as i32);
        }

        power
    }
}
