pub mod map;
pub mod state;
pub mod ui;

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
}
