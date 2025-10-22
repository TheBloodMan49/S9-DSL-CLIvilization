pub mod map;
pub mod state;
pub mod ui;
mod utils;

use self::state::GameState;
use self::ui::draw_ui;

pub struct Game {
    state: GameState,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) -> std::io::Result<()> {
        terminal.draw(|frame| draw_ui(frame, &self.state))?;
        Ok(())
    }

    // New: handle key events for seed input and editing
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        match key.code {
            // toggle seed edit mode
            KeyCode::Char('s') => {
                self.state.toggle_seed_edit();
            }
            // submit seed and rebuild map
            KeyCode::Enter => {
                if self.state.seed_editing {
                    self.state.submit_seed();
                }
            }
            // backspace while editing
            KeyCode::Backspace => {
                if self.state.seed_editing {
                    self.state.backspace_seed();
                }
            }
            // character input while editing
            KeyCode::Char(c) => {
                if self.state.seed_editing {
                    self.state.add_seed_char(c);
                }
            }
            _ => {}
        }
    }
}
