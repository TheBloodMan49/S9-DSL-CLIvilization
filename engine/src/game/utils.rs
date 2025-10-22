use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use ratatui::prelude::{Color, Line, Span, Style};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn hash_tmb(text: String) -> u32 {
    let mut hash: u32 = 2166136261; // FNV offset basis

    for byte in text.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619); // FNV prime
    }

    hash
}

pub fn draw_color_test(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
    // Build 16x16 grid of indexed colors (0..=255)
    let cols = 16;
    let mut lines: Vec<Line> = Vec::new();
    for row in 0..16 {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..cols {
            let idx = (row * cols + col) as u8;
            // Each cell shows the index as 3 chars with background set to the indexed color
            let text = format!("{:>3}", idx);
            let style = Style::default().bg(Color::Indexed(idx)).fg(Color::Reset);
            spans.push(Span::styled(text, style));
            // add a small spacer between cells
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }

    terminal.draw(|f| {
        let size = f.size();
        let block = Paragraph::new(lines.clone())
            .block(Block::default().title("Terminal 256-color test (press any key to exit)").borders(Borders::ALL));
        f.render_widget(block, size);
    })?;

    Ok(())
}

pub fn cleanup_term(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
