mod ast;
mod game;

use anyhow::Context;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
    event::{self, Event, KeyCode, KeyModifiers},
};
use ratatui::{
    prelude::*,
    backend::CrosstermBackend,
};
use std::io;
use clap::Parser;
use crate::game::ui::{cleanup_term, draw_color_test_256, draw_color_test_rgb};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run a color test screen instead of the game
    #[arg(long, default_value = "", value_parser = ["", "256", "rgb"])]
    test_color: String,
}

fn main() -> Result<()> {
    // Use clap to parse a --test-color flag for testing color schemes
    let matches = Args::parse();

    // Setup terminal
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // If test_color was requested, show the color test and exit on any key press
    match matches.test_color.as_str() {
        "256" => {
            draw_color_test_256(&mut terminal)?;
            // Wait for any key press
            event::read()?;
            cleanup_term(&mut terminal)?;
            return Ok(());
        }
        "rgb" => {
            draw_color_test_rgb(&mut terminal)?;
            // Wait for any key press
            event::read()?;
            cleanup_term(&mut terminal)?;
            return Ok(());
        }
        _ => {}
    }

    // Create a game instance
    let mut game = game::Game::new();

    // Game loop
    loop {
        // Draw frame
        game.run(&mut terminal)?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Quit on Ctrl+Q
                if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                // Forward other keys to game handler
                game.handle_key(key);
            }
        }
    }

    // Cleanup
    cleanup_term(&mut terminal)?;
    Ok(())
}
