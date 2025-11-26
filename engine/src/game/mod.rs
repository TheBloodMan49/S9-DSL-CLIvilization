pub mod ai;
pub mod map;
pub mod state;
pub mod ui;
pub mod utils;

use self::state::GameState;
use self::ui::draw_ui;
use crate::game::ui::UiConfig;
use crate::game::utils::{str_to_color, write_to_file};
use anyhow::Context;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    Normal,
    EditingSeed,
    CameraMode,
    ActionEditing,
    PopupOpen,
}

// ===== AI trait + simple RandomAI implementation =====

/// AI trait: implement to allow programmatic players. The AI receives a read-only view of the game
/// state and the index of the civilization it controls.
pub trait Ai: Send {
    /// Return an action string to perform, or None to indicate "no more actions / end turn".
    fn select_action(&mut self, view: &AiView, civ_index: usize) -> Option<String>;

    /// When a popup is opened, provide the textual input (e.g. "1" or a name) to submit the popup.
    fn select_popup_input(
        &mut self,
        _view: &AiView,
        _civ_index: usize,
        popup: &state::Popup,
    ) -> String {
        // Default: pick the first choice if any
        if popup.choices.is_empty() {
            popup.input.clone()
        } else {
            "1".to_string()
        }
    }
}

/// Very small random AI used as an example implementation.
pub struct RandomAi {
    rng: SmallRng,
}

impl RandomAi {
    pub fn new() -> Self {
        // Seed SmallRng from a random u64
        let mut tr = rand::thread_rng();
        let seed: u64 = tr.random();
        let rng = SmallRng::seed_from_u64(seed);
        Self { rng }
    }
}

impl Ai for RandomAi {
    fn select_action(&mut self, view: &AiView, civ_index: usize) -> Option<String> {
        // Build a list of candidate actions
        let mut actions: Vec<String> = Vec::new();
        // end is always allowed
        actions.push("end".to_string());

        // build options
        for b in &view.buildings {
            actions.push(format!("build {}", b.to_lowercase()));
        }
        // hire options
        for u in &view.units {
            actions.push(format!("hire {}", u.to_lowercase()));
        }
        // attack options (other players)
        for (i, p) in view.players.iter().enumerate() {
            if i != civ_index {
                actions.push(format!("attack {}", p.name.to_lowercase()));
            }
        }

        if actions.is_empty() {
            return Some("end".into());
        }

        let idx = self.rng.gen_range(0..actions.len());
        Some(actions.swap_remove(idx))
    }

    fn select_popup_input(
        &mut self,
        _view: &AiView,
        _civ_index: usize,
        popup: &state::Popup,
    ) -> String {
        if popup.choices.is_empty() {
            // no choices, return empty input
            String::new()
        } else {
            let idx = self.rng.gen_range(0..popup.choices.len());
            // return 1-based index as string
            (idx + 1).to_string()
        }
    }
}

pub struct Game {
    state: GameState,
    ui_state: UiState,
    ui_config: UiConfig,
    // One AI slot per civilization; None means human / not driven by AI.
    ais: Vec<Option<Box<dyn Ai>>>,
}

// Lightweight view passed to AIs to avoid borrows of self
#[derive(Clone)]
pub struct AiPlayerView {
    pub name: String,
    pub resources: i32,
    pub buildings: usize,
    pub units: usize,
}

pub struct AiView {
    pub turn: i32,
    pub player_turn: usize,
    pub players: Vec<AiPlayerView>,
    pub buildings: Vec<String>,
    pub units: Vec<String>,
    pub seed: String,
}

impl Game {
    pub fn new() -> Self {
        let state = GameState::new();
        let mut ais: Vec<Option<Box<dyn Ai>>> = Vec::new();
        ais.resize_with(state.civilizations.len(), || None);
        Self {
            state,
            ui_state: UiState::Normal,
            ui_config: UiConfig {
                color: ratatui::style::Color::Rgb(255, 255, 255),
            },
            ais,
        }
    }

    pub fn from_file(config_path: &str) -> anyhow::Result<Self> {
        // Read file
        let contents = std::fs::read_to_string(config_path)
            .context(format!("failed to read config file `{config_path}`"))?;

        Self::from_string(&contents)
    }

    pub fn from_string(config_string: &str) -> anyhow::Result<Self> {
        // Parse JSON into AST model
        let model: crate::ast::Model =
            serde_json::from_str(config_string).context("failed to parse config JSON")?;

        // Start from default game state
        let mut game = Game::new();

        // Walk sections and apply relevant settings (only Game section is needed for now)
        for section in model.sections {
            match section {
                crate::ast::Section::Game(g) => {
                    // ui color
                    game.ui_config.color = str_to_color(&g.ui_color);

                    // map settings
                    let map = map::GameMap::new(
                        g.seed.clone().unwrap_or("pokemon".into()),
                        g.map_x as usize,
                        g.map_y as usize,
                    );
                    game.state.map = map;

                    // current turn
                    game.state.turn = g.current_turn.cast_signed();
                }
                crate::ast::Section::BuildingDefArray(bda) => {
                    game.state.buildings = bda.buildings;
                }
                crate::ast::Section::UnitDefArray(uda) => {
                    game.state.units = uda.units;
                }
                crate::ast::Section::Cities(cities) => {
                    // Load cities into civilizations
                    game.state.civilizations = cities
                        .cities
                        .into_iter()
                        .map(|city| state::Civilization {
                            resources: state::Resources { ressources: 100 },
                            city,
                            alive: true,
                            constructions: Vec::new(),
                            recruitments: Vec::new(),
                        })
                        .collect();
                    // Ensure AI slots match civilizations
                    game.ais = Vec::new();
                    game.ais
                        .resize_with(game.state.civilizations.len(), || None);
                }
                crate::ast::Section::VictoryConditions(vc) => {
                    game.state.nb_turns = vc.nb_turns;
                    game.state.resources_spent = vc.resources_spent;
                }
            }
        }

        Ok(game)
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> std::io::Result<()> {
        terminal.draw(|frame| draw_ui(frame, &mut self.state, &self.ui_config))?;
        Ok(())
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        // If the game is over, prevent game actions but still allow zoom and entering camera mode.
        if self.state.game_over {
            match key.code {
                KeyCode::Char('z') | KeyCode::Char('Z') => {
                    self.state.cycle_zoom();
                    return;
                }
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    // allow entering camera mode to move around
                    self.state.toggle_camera_mode();
                    self.ui_state = UiState::CameraMode;
                    return;
                }
                _ => {
                    // ignore all other keys when game is over
                    return;
                }
            }
        }

        match self.ui_state {
            UiState::Normal => {
                match key.code {
                    // enter seed editing mode
                    KeyCode::Char('s') => {
                        self.state.toggle_seed_edit();
                        self.ui_state = UiState::EditingSeed;
                    }
                    // Pick random seed
                    KeyCode::Char('r') => {
                        self.state.map =
                            map::GameMap::new_random(self.state.map.width, self.state.map.height);
                    }
                    KeyCode::Char('v' | 'V') => {
                        self.state.toggle_camera_mode();
                        self.ui_state = UiState::CameraMode;
                    }
                    KeyCode::Char('a') => {
                        // start typing an action
                        self.state.start_action_input();
                        self.ui_state = UiState::ActionEditing;
                    }
                    KeyCode::Char('z' | 'Z') => {
                        self.state.cycle_zoom();
                    }
                    KeyCode::Char('w') => {
                        // Write map to file
                        let filename = format!("map_{}.txt", self.state.map.seed);
                        // Open file and get map string
                        let map_string = self.state.map.to_string();
                        write_to_file(&filename, &map_string).expect("TODO: panic message");
                    }
                    _ => {
                        // other global key handling could go here
                    }
                }
            }
            UiState::EditingSeed => {
                match key.code {
                    // submit seed and exit editing
                    KeyCode::Enter => {
                        // GameState::submit_seed already clears seed_editing
                        self.state.submit_seed();
                        self.ui_state = UiState::Normal;
                    }
                    // cancel editing with Esc (stop editing, don't change seed)
                    KeyCode::Esc => {
                        if self.state.seed_editing {
                            // stop editing; keep seed_input as-is (no submit)
                            self.state.toggle_seed_edit();
                        }
                        self.ui_state = UiState::Normal;
                    }
                    // backspace while editing
                    KeyCode::Backspace => {
                        self.state.backspace_seed();
                    }
                    // character input while editing (including 's')
                    KeyCode::Char(c) => {
                        self.state.add_seed_char(c);
                    }
                    _ => {}
                }
            }
            UiState::CameraMode => {
                match key.code {
                    // exit camera mode
                    KeyCode::Char('v' | 'V') | KeyCode::Esc => {
                        self.state.toggle_camera_mode();
                        self.ui_state = UiState::Normal;
                    }
                    // camera movement
                    KeyCode::Char('z' | 'Z') => {
                        self.state.move_camera(0, -1);
                    }
                    KeyCode::Char('s' | 'S') => {
                        self.state.move_camera(0, 1);
                    }
                    KeyCode::Char('q' | 'Q') => {
                        self.state.move_camera(-1, 0);
                    }
                    KeyCode::Char('d' | 'D') => {
                        self.state.move_camera(1, 0);
                    }
                    _ => {}
                }
            }
            UiState::ActionEditing => {
                match key.code {
                    KeyCode::Enter => {
                        // submit action, may open a popup
                        let opened = self.state.submit_action();
                        self.ui_state = if opened {
                            UiState::PopupOpen
                        } else {
                            UiState::Normal
                        };
                    }
                    KeyCode::Esc => {
                        self.state.action_editing = false;
                        self.ui_state = UiState::Normal;
                    }
                    KeyCode::Backspace => {
                        self.state.backspace_action();
                    }
                    KeyCode::Char(c) => {
                        self.state.add_action_char(c);
                    }
                    _ => {}
                }
            }
            UiState::PopupOpen => match key.code {
                KeyCode::Enter => {
                    self.state.submit_popup();
                    self.ui_state = UiState::Normal;
                }
                KeyCode::Esc => {
                    self.state.close_popup();
                    self.ui_state = UiState::Normal;
                }
                KeyCode::Backspace => {
                    if let Some(p) = &mut self.state.popup {
                        p.input.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(p) = &mut self.state.popup {
                        p.input.push(c);
                    }
                }
                _ => {}
            },
        }
    }

    // ===== Headless / programmatic API =====

    /// Apply an action by string. Returns true if this resulted in a popup opening (requires further input).
    pub fn apply_action(&mut self, action: &str) -> bool {
        // prepare action input like interactive mode would
        log::info!("apply_action called: {action}");
        self.state.action_input = action.to_string();
        self.state.action_editing = true;
        let opened = self.state.submit_action();
        // update UI state to reflect popup if needed
        self.ui_state = if opened {
            UiState::PopupOpen
        } else {
            UiState::Normal
        };
        opened
    }

    /// Provide input for an open popup (the text entered by user) and submit it
    /// Returns true if a popup was present and processed.
    pub fn submit_popup_input(&mut self, input: &str) -> bool {
        if self.state.popup.is_none() {
            return false;
        }
        log::info!("submit_popup_input: {input}");
        if let Some(p) = &mut self.state.popup {
            p.input = input.to_string();
        }
        self.state.submit_popup();
        self.ui_state = UiState::Normal;
        true
    }

    /// Advance the turn as if the current player ended their turn
    pub fn step(&mut self) {
        self.state.player_turn = (self.state.player_turn + 1) % self.state.civilizations.len();
        if self.state.player_turn == 0 {
            self.state.turn += 1;
        }
    }

    /// Borrow the inner state for read-only inspection
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Borrow the inner state mutably
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Produce a compact JSON value snapshot describing key game state. Uses `serde_json::Value`.
    pub fn snapshot_value(&self) -> serde_json::Value {
        let players: Vec<serde_json::Value> = self
            .state
            .civilizations
            .iter()
            .map(|c| {
                serde_json::json!({
                    "name": c.city.name,
                    "resources": c.resources.ressources,
                    "buildings": c.city.buildings.elements.len(),
                    "units": c.city.units.units.len(),
                })
            })
            .collect();

        serde_json::json!({
            "turn": self.state.turn,
            "player_turn": self.state.player_turn,
            "players": players,
            "seed": self.state.map.seed,
        })
    }

    /// Register an AI instance to control the civilization at `civ_index`.
    pub fn register_ai(&mut self, civ_index: usize, ai: Box<dyn Ai>) {
        if civ_index >= self.ais.len() {
            // grow to fit
            self.ais.resize_with(civ_index + 1, || None);
        }
        self.ais[civ_index] = Some(ai);
        log::info!("Registered AI for civ {civ_index}");
    }

    /// Return a list of plausible actions for the civilization index (lowercased strings as used by the parser)
    pub fn ai_possible_actions(&self, civ_index: usize) -> Vec<String> {
        let mut actions: Vec<String> = Vec::new();
        actions.push("end".to_string());
        for b in &self.state.buildings {
            actions.push(format!("build {}", b.name.to_lowercase()));
        }
        for u in &self.state.units {
            actions.push(format!("hire {}", u.name.to_lowercase()));
        }
        for (i, civ) in self.state.civilizations.iter().enumerate() {
            if i != civ_index {
                actions.push(format!("attack {}", civ.city.name.to_lowercase()));
            }
        }
        log::debug!("ai_possible_actions for civ {} => {} actions", civ_index, actions.len());
        actions
    }

    /// Build a lightweight snapshot of the state for AI decision making.
    pub fn make_ai_view(&self) -> AiView {
        let players = self
            .state
            .civilizations
            .iter()
            .map(|c| AiPlayerView {
                name: c.city.name.clone(),
                resources: c.resources.ressources,
                buildings: c.city.buildings.elements.len(),
                units: c.city.units.units.len(),
            })
            .collect();

        let buildings = self
            .state
            .buildings
            .iter()
            .map(|b| b.name.clone())
            .collect();
        let units = self.state.units.iter().map(|u| u.name.clone()).collect();

        AiView {
            turn: self.state.turn,
            player_turn: self.state.player_turn,
            players,
            buildings,
            units,
            seed: self.state.map.seed.clone(),
        }
    }

    /// If the current player is controlled by an AI, make that AI play until it ends its turn.
    /// This method will repeatedly ask the AI for actions and apply them.
    pub fn run_ai_for_current_player(&mut self) {
        // safety cap to avoid infinite loops from buggy AIs
        const MAX_ACTIONS: usize = 256;
        let mut actions_done = 0usize;

        loop {
            if actions_done >= MAX_ACTIONS {
                log::warn!("AI action loop reached MAX_ACTIONS ({MAX_ACTIONS})");
                break;
            }
            let civ_idx = self.state.player_turn;
            // if there is no AI registered for this civ, stop
            if civ_idx >= self.ais.len() {
                log::debug!("No AI registered for civ {civ_idx} (out of range)");
                break;
            }
            if self.ais[civ_idx].is_none() {
                log::debug!("No AI registered for civ {civ_idx}");
                break;
            }

            // Only run AI if the civilization is actually flagged AI in the city definition
            if let Some(civ) = self.state.civilizations.get(civ_idx) {
                use crate::ast::PlayerType;
                if !matches!(civ.city.player_type, PlayerType::AI) {
                    log::debug!("Civ {civ_idx} is not marked as AI; skipping");
                    break;
                }
            } else {
                log::warn!("Civ {civ_idx} not found in state");
                break;
            }

            // build view snapshot
            let view = self.make_ai_view();

            // ask AI for action
            let action_opt = {
                let ai_mut = self.ais[civ_idx].as_mut().unwrap();
                ai_mut.select_action(&view, civ_idx)
            };

            if let Some(action) = action_opt {
                log::info!("AI selected action for civ {civ_idx}: {action}");
                let opened = self.apply_action(&action);
                if opened && let Some(popup) = &self.state.popup {
                    log::info!("AI opened popup: {}", popup.title);
                    let popup_clone = popup.clone();
                    let view2 = self.make_ai_view();
                    let input = {
                        let ai_mut = self.ais[civ_idx].as_mut().unwrap();
                        ai_mut.select_popup_input(&view2, civ_idx, &popup_clone)
                    };
                    log::info!("AI popup input for civ {civ_idx}: {input}");
                    self.submit_popup_input(&input);
                }
            } else {
                log::info!("AI returned no action for civ {civ_idx}; ending turn");
                self.step();
            }

            actions_done += 1;

            let new_civ = self.state.player_turn;
            if new_civ >= self.ais.len() || self.ais[new_civ].is_none() {
                log::debug!("Next civ {new_civ} has no AI, stopping AI loop");
                break;
            }
        }
    }
}
