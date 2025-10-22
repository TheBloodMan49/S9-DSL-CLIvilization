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
        }
    }
}
