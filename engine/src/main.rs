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
    #[arg(long, value_parser = ["256", "rgb"])]
    test_color: Option<String>,

    /// Load config from file
    #[arg(long)]
    config: Option<String>,

    /// Dump config blob
    #[arg(long)]
    blob: bool,
}

fn main() -> Result<()> {
    // Use clap to parse a --test-color flag for testing color schemes
    let matches = Args::parse();

    // Get the config blob
    let blob = option_env!("CONFIG_BLOB");

    if matches.blob {
        if let Some(blob_str) = blob {
            println!("{}", blob_str);
        } else {
            println!("This binary does not contain a blob.");
        }

        return Ok(());
    }

    // Setup terminal
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // If test_color was requested, show the color test and exit on any key press
    if let Some(test_type) = matches.test_color {
        match test_type.as_str() {
            "256" => draw_color_test_256(&mut terminal)?,
            "rgb" => draw_color_test_rgb(&mut terminal)?,
            _ => unreachable!(),
        }
        // Wait for a key press
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }
        cleanup_term(&mut terminal)?;
        return Ok(());
    }
    
    // Load config if provided
    let mut game = if let Some(config_path) = matches.config {
        game::Game::from_file(&config_path)?
    } else {
        if let Some(blob_str) = blob {
            game::Game::from_string(blob_str)?
        } else {
            game::Game::new()
        }
    };

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
