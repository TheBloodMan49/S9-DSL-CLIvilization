use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use serde::Deserialize;
use std::{collections::HashMap, fs, io};

#[derive(Debug, Deserialize)]
struct Model {
    declarations: Vec<Decl>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Decl {
    TileDef {
        name: String,
        glyph: String,
        nameStr: String,
        cost: i32,
    },
    MapDef {
        name: String,
        width: usize,
        height: usize,
        rows: Vec<Row>,
    },
    PlayerDef {
        id: String,
        name: String,
        units: Option<Vec<UnitEntry>>,
    },
}

#[derive(Debug, Deserialize, Clone)]
struct Row {
    index: usize,
    pattern: String,
}

#[derive(Debug, Deserialize, Clone)]
struct UnitEntry {
    kind: String,
    x: usize,
    y: usize,
}

#[derive(Clone, Debug)]
struct Tile {
    glyph: char,
    name: String,
    cost: i32,
}

#[derive(Clone, Debug)]
struct GameMap {
    width: usize,
    height: usize,
    tiles: Vec<Vec<Tile>>,
}

fn load_game_map() -> Result<GameMap> {
    let data = fs::read_to_string("config.json").context("read config.json")?;
    let model: Model = serde_json::from_str(&data).context("parse json")?;

    let mut palette: HashMap<char, Tile> = HashMap::new();
    let mut map_width = 0usize;
    let mut map_height = 0usize;
    let mut map_rows: Vec<String> = Vec::new();

    for d in &model.declarations {
        match d {
            Decl::TileDef {
                name,
                glyph,
                nameStr,
                cost,
            } => {
                let ch = glyph.chars().find(|c| !c.is_whitespace()).unwrap_or('?');
                palette.insert(
                    ch,
                    Tile {
                        glyph: ch,
                        name: nameStr.clone(),
                        cost: *cost,
                    },
                );
            }
            Decl::MapDef {
                width,
                height,
                rows,
                ..
            } => {
                map_width = *width;
                map_height = *height;
                let mut ordered = rows.clone();
                ordered.sort_by_key(|r| r.index);
                for r in ordered {
                    map_rows.push(r.pattern);
                }
            }
            _ => {}
        }
    }

    let mut grid: Vec<Vec<Tile>> =
        vec![vec![Tile { glyph: '?', name: "Unknown".into(), cost: 1 }; map_width]; map_height];

    for (y, row) in map_rows.iter().enumerate().take(map_height) {
        for (x, ch) in row.chars().enumerate().take(map_width) {
            if let Some(t) = palette.get(&ch) {
                grid[y][x] = t.clone();
            } else {
                grid[y][x] = Tile {
                    glyph: ch,
                    name: format!("Unknown({})", ch),
                    cost: 1,
                };
            }
        }
    }

    Ok(GameMap {
        width: map_width,
        height: map_height,
        tiles: grid,
    })
}

fn main() -> Result<()> {
    // Load map
    let map = load_game_map()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = run_app(&mut terminal, map);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, map: GameMap) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                .split(size);

            let map_str = render_map(&map);
            let para = Paragraph::new(map_str)
                .block(Block::default().borders(Borders::ALL).title("CLIvilization"))
                .style(Style::default().fg(Color::White));
            f.render_widget(para, chunks[0]);

            let footer = Paragraph::new(Line::from(vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to quit."),
            ]));
            f.render_widget(footer, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
                && key.code == KeyCode::Char('q') {
                    break;
                }
    }
    Ok(())
}

fn render_map(map: &GameMap) -> String {
    let mut s = String::new();
    for row in &map.tiles {
        for t in row {
            s.push(t.glyph);
        }
        s.push('\n');
    }
    s
}
