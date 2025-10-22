mod game;
mod ast;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, prelude::*};
use std::io;

use crate::game::utils::{cleanup_term, draw_color_test};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = false)]
    test_color: bool,
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

    // If test_color was requested, show the color test and exit without launching the game
    if matches.test_color {
        draw_color_test(&mut terminal)?;
        // Wait for any key
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }
        // Cleanup and exit
        cleanup_term(&mut terminal)?;
        return Ok(());
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
