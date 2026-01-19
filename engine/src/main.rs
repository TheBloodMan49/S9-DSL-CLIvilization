mod ast;
mod game;
mod logger;

use crate::game::ui::{cleanup_term, draw_color_test_256, draw_color_test_rgb};
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, prelude::*};
use std::io;
use log::warn;
use crate::game::ai::LlmAi;

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

    /// Run in headless mode, for automated testing or AI play
    #[arg(long)]
    headless: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file (optional)
    if dotenvy::dotenv().is_err() {
        warn!("Warning: No .env file found. Environment variables must be set manually.");
        warn!("Required for AI: OPENAI_KEY or OPENAI_API_KEY");
        warn!("Optional: OPENAI_BASE_URL or OPENAI_API_BASE, AI_MODEL, LOG_LEVEL");
    }

    // Initialize file logger as early as possible so any startup errors are logged
    logger::init("output/game.log").context("failed to initialize logger")?;

    // Install a panic hook so unexpected panics are recorded to the log file
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Unknown panic payload",
            },
        };
        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}", loc.file(), loc.line())
        } else {
            "unknown location".to_string()
        };
        log::error!("PANIC at {location}: {payload}");
    }));

    log::info!("Starting clivilization-engine");

    // LLM-backed AI will be registered per-civ below

    // Use clap to parse a --test-color flag for testing color schemes
    let matches = Args::parse();

    // Get the config blob
    let blob = option_env!("CONFIG_BLOB");

    if matches.blob {
        if let Some(blob_str) = blob {
            println!("{blob_str}");
        } else {
            println!("This binary does not contain a blob.");
        }

        return Ok(());
    }

    // Load config if provided
    log::info!("Loading game configuration");
    let mut game = if let Some(config_path) = matches.config {
        log::info!("Loading config from {config_path}");
        game::Game::from_file(&config_path)?
    } else if let Some(blob_str) = blob {
        log::info!("Loading config from embedded blob");
        game::Game::from_string(blob_str)?
    } else {
        log::info!("Creating default game instance");
        game::Game::new()
    };

    // If headless, run a simple stdin-driven loop and avoid initializing terminal or crossterm
    if matches.headless {
        log::info!("Starting in headless mode");
        
        use std::io::{BufRead, BufReader};

        let stdin = io::stdin();
        let reader = BufReader::new(stdin);

        // Register AIs for headless mode
        // First collect indices to avoid borrowing `game` immutably while mutating it
        let ai_model = std::env::var("AI_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());
        let mut ai_indices: Vec<usize> = Vec::new();
        for (i, civ) in game.state().civilizations.iter().enumerate() {
            if matches!(civ.city.player_type, ast::PlayerType::AI) {
                ai_indices.push(i);
            }
        }
        for i in ai_indices {
            game.register_ai(i, Box::new(LlmAi::new(Box::leak(ai_model.clone().into_boxed_str()))));
            log::info!("Registered AI for civ {} (headless) with model {}", i, ai_model);
        }

        // Emit initial snapshot
        let snap = game.snapshot_value();
        println!("{}", serde_json::to_string(&snap)?);

        // If current player is AI, run it immediately
        game.run_ai_for_current_player();
        println!("{}", serde_json::to_string(&game.snapshot_value())?);

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut parts = trimmed.split_whitespace();
            match parts.next().unwrap_or("") {
                "quit" | "exit" => break,
                "snapshot" => {
                    let snap = game.snapshot_value();
                    println!("{}", serde_json::to_string(&snap)?);
                }
                "step" => {
                    game.step();
                    // after stepping, if new player is AI, run it
                    game.run_ai_for_current_player();
                    let snap = game.snapshot_value();
                    println!("{}", serde_json::to_string(&snap)?);
                }
                "apply" => {
                    // apply rest of line as action
                    let action = parts.collect::<Vec<&str>>().join(" ");

                    // If current player is AI, refuse to apply human actions
                    let civ_idx = game.state().player_turn;
                    if let Some(civ) = game.state().civilizations.get(civ_idx)
                        && matches!(civ.city.player_type, ast::PlayerType::AI) {
                            log::warn!("Headless apply refused: it's AI's turn for civ {civ_idx}");
                            println!("{{\"error\":\"cannot apply action: it's AI's turn\"}}");
                            continue;
                        }

                    log::info!("Applying action from stdin: {action}");
                    let opened = game.apply_action(&action);
                    if opened {
                        // print snapshot with popup
                        let mut v = game.snapshot_value();
                        if let Some(p) = &game.state().popup {
                            v["popup"] = serde_json::json!({"title": p.title, "prompt": p.prompt, "choices": p.choices});
                        }
                        println!("{}", serde_json::to_string(&v)?);
                    } else {
                        // action applied; if this caused the player to end and next is AI, run it
                        game.run_ai_for_current_player();
                        println!("{}", serde_json::to_string(&game.snapshot_value())?);
                    }
                }
                "popup" => {
                    // submit popup input (rest of line)
                    let input = parts.collect::<Vec<&str>>().join(" ");

                    // If current player is AI, refuse to submit popup input
                    let civ_idx = game.state().player_turn;
                    if let Some(civ) = game.state().civilizations.get(civ_idx)
                        && matches!(civ.city.player_type, ast::PlayerType::AI) {
                            log::warn!("Headless popup submit refused: it's AI's turn for civ {civ_idx}");
                            println!("{{\"error\":\"cannot submit popup: it's AI's turn\"}}");
                            continue;
                        }

                    log::info!("Submitting popup input from stdin: {input}");
                    let _processed = game.submit_popup_input(&input);
                    // After popup submission, AI may have to act (e.g., popup closed)
                    game.run_ai_for_current_player();
                    let snap = game.snapshot_value();
                    println!("{}", serde_json::to_string(&snap)?);
                }
                other => {
                    // Unknown command -> respond with a helpful message and snapshot
                    log::warn!("Unknown headless command: {other}");
                    eprintln!("Unknown command: {trimmed}");
                    let snap = game.snapshot_value();
                    println!("{}", serde_json::to_string(&snap)?);
                }
            }
        }

        return Ok(());
    }

    log::info!("Starting UI mode");

    // Setup terminal (only for non-headless mode)
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // If test_color was requested, show the color test and exit on any key press
    if let Some(test_type) = matches.test_color {
        // Wait for a key press
        let mut offset = 0;
        loop {
            match test_type.as_str() {
                "256" => draw_color_test_256(&mut terminal)?,
                "rgb" => draw_color_test_rgb(&mut terminal, offset)?,
                _ => unreachable!(),
            }
            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
            {
                if key.code == KeyCode::Char('d') {
                    offset += 1;
                } else {
                    break;
                }
            }
        }
        cleanup_term(&mut terminal)?;
        return Ok(());
    }

    // Register AIs for UI mode as well so the UI can auto-play AI turns
    if !matches.headless {
        use crate::game::ai::LlmAi;
        let ai_model = std::env::var("AI_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());
        let mut ai_indices: Vec<usize> = Vec::new();
        for (i, civ) in game.state().civilizations.iter().enumerate() {
            if matches!(civ.city.player_type, ast::PlayerType::AI) {
                ai_indices.push(i);
            }
        }
        for i in ai_indices {
            game.register_ai(i, Box::new(LlmAi::new(Box::leak(ai_model.clone().into_boxed_str()))));
            log::info!("Registered LlmAi for civ {} (UI) with model {}", i, ai_model);
        }
    }

    // Game loop
    loop {
        // Check if current player is AI and set the flag (but don't run yet)
        let is_ai_turn = if let Some(civ) = game.state().civilizations.get(game.state().player_turn) {
            matches!(civ.city.player_type, ast::PlayerType::AI)
        } else {
            false
        };
        
        if is_ai_turn {
            game.state_mut().ai_thinking = true;
            game.state_mut().action_input.clear();
            game.state_mut().action_editing = false;
        }

        // Draw frame (this will show the AI thinking popup if ai_thinking is true)
        game.run(&mut terminal)?;

        // Now run the AI if it's their turn (after the popup has been drawn)
        if is_ai_turn {
            game.run_ai_for_current_player();
        }

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            // Quit on Ctrl+Q
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }

            // If it's currently an AI player's turn, ignore user input (prevent playing on AI's turn)
            if let Some(civ) = game.state().civilizations.get(game.state().player_turn)
                && matches!(civ.city.player_type, ast::PlayerType::AI) {
                    log::debug!("User input ignored because it's AI's turn: {key:?}");
                    continue;
                }

            // Forward other keys to game handler
            game.handle_key(key);
        }
    }

    // Cleanup
    cleanup_term(&mut terminal)?;
    Ok(())
}
