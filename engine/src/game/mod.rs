pub mod map;
pub mod state;
pub mod ui;
pub mod utils;

use self::state::GameState;
use self::ui::draw_ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    Normal,
    EditingSeed,
    CameraMode,
}

pub struct Game {
    state: GameState,
    ui_state: UiState,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            ui_state: UiState::Normal,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> std::io::Result<()> {
        terminal.draw(|frame| draw_ui(frame, &self.state))?;
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
                        self.state.map = map::GameMap::new_random();
                        self.state.seed_input = self.state.map.seed.clone();
                    }
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        self.state.toggle_camera_mode();
                        self.ui_state = UiState::CameraMode;
                    }
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        self.state.cycle_zoom();
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
        }
    }
}
