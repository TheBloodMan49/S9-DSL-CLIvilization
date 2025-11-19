pub mod map;
pub mod state;
pub mod ui;
pub mod utils;

use anyhow::Context;
use crate::game::ui::UiConfig;
use crate::game::utils::{str_to_color, write_to_file};
use self::state::GameState;
use self::ui::draw_ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    Normal,
    EditingSeed,
    CameraMode,
    ActionEditing,
    PopupOpen,
}

pub struct Game {
    state: GameState,
    ui_state: UiState,
    ui_config: UiConfig
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            ui_state: UiState::Normal,
            ui_config: UiConfig {
                color: ratatui::style::Color::Rgb(255, 255, 255),
            },
        }
    }

    pub fn from_file(config_path: &str) -> anyhow::Result<Self> {

        // Read file
        let contents = std::fs::read_to_string(config_path)
            .context(format!("failed to read config file `{}`", config_path))?;

        Self::from_string(&contents)
    }
    
    pub fn from_string(config_string: &str) -> anyhow::Result<Self> {
        
        // Parse JSON into AST model
        let model: crate::ast::Model = serde_json::from_str(config_string)
            .context("failed to parse config JSON")?;

        // Start from default game state
        let mut game = Game::new();

        // Walk sections and apply relevant settings (only Game section is needed for now)
        for section in model.sections.into_iter() {
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
                    game.state.turn = g.current_turn as i32;
                }
                crate::ast::Section::BuildingDefArray(bda) => {
                    game.state.buildings = bda.buildings;
                }
                crate::ast::Section::UnitDefArray(uda) => {
                    game.state.units = uda.units;
                }
                crate::ast::Section::Cities(cities) => {
                    // Load cities into civilizations
                    game.state.civilizations = cities.cities.into_iter().map(|city| {
                        state::Civilization {
                            resources: state::Resources { ressources: 100 },
                            city,
                        }
                    }).collect();
                }
                crate::ast::Section::VictoryConditions(_vc) => {
                    game.state.nb_turns = _vc.nb_turns;
                    game.state.resources_spent = _vc.resources_spent;
                }
            }
        }

        Ok(game)
    }
    
    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> std::io::Result<()> {
        terminal.draw(|frame| draw_ui(frame, &self.state, &self.ui_config))?;
        Ok(())
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

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
                        self.state.map = map::GameMap::new_random(self.state.map.width, self.state.map.height);
                    }
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        self.state.toggle_camera_mode();
                        self.ui_state = UiState::CameraMode;
                    }
                    KeyCode::Char('a') => {
                        // start typing an action
                        self.state.start_action_input();
                        self.ui_state = UiState::ActionEditing;
                    }
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
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
                    KeyCode::Char('v') | KeyCode::Char('V') | KeyCode::Esc => {
                        self.state.toggle_camera_mode();
                        self.ui_state = UiState::Normal;
                    }
                    // camera movement
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        self.state.move_camera(0, -1);
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        self.state.move_camera(0, 1);
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        self.state.move_camera(-1, 0);
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
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
                        self.ui_state = if opened { UiState::PopupOpen } else { UiState::Normal };
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
            UiState::PopupOpen => {
                match key.code {
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
                }
            }
        }
    }
}
