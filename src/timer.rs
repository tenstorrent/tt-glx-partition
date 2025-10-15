use std::io::{self, stdout};
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Block, Borders},
    symbols,
    layout::{Alignment, Flex},
};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    // Main loop
    loop {
        terminal.draw(|frame| {
            let area = frame.size();

            // Create the clock text with current time
            let now = chrono::Local::now();
            let time_text = now.format("%H:%M:%S").to_string();

            // Create a paragraph with the time
            let clock = Paragraph::new(time_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White))
                        .title(" Clock ")
                        .title_alignment(Alignment::Center)
                );

            frame.render_widget(clock, area);
        })?;

        // Handle events and timing
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}