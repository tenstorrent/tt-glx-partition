use crossterm::{
    event::{self, Event, KeyCode},
};
use ratatui::{
    backend::{Backend},
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame, Terminal,
};
use std::time::{Duration, Instant};
use ratatui::style::Stylize;
use std::io;

const TT_GREEN: Color = Color::Rgb(30, 181, 125);

pub struct App {
    vertical_scroll_state: ScrollbarState,
    horizontal_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    horizontal_scroll: usize,
    content_height: usize,
    content_width: usize,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let content_height = 100; // Simulate long content
        let content_width = 100;  // Simulate wide content
        let vertical_scroll_state = ScrollbarState::new(content_height).viewport_content_length(10);
        let horizontal_scroll_state = ScrollbarState::new(content_width).viewport_content_length(3);
        Self {
            vertical_scroll_state,
            horizontal_scroll_state,
            vertical_scroll: 0,
            horizontal_scroll: 0,
            content_height,
            content_width,
            should_quit: false,
        }
    }

    fn on_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    fn on_down(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
        if self.vertical_scroll >= self.content_height {
            self.vertical_scroll = self.content_height.saturating_sub(1);
        }
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    fn on_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
        self.horizontal_scroll_state = self.horizontal_scroll_state.position(self.horizontal_scroll);
    }

    fn on_right(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
        if self.horizontal_scroll >= self.content_width {
            self.horizontal_scroll = self.content_width.saturating_sub(1);
        }
        self.horizontal_scroll_state = self.horizontal_scroll_state.position(self.horizontal_scroll);
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Up => self.on_up(),
            KeyCode::Down => self.on_down(),
            KeyCode::Left => self.on_left(),
            KeyCode::Right => self.on_right(),
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.on_up();
                }
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.on_down();
                }
            }
            _ => {}
        }
    }

}

fn render_header(frame: &mut Frame, area: Rect, current_time: String) {
    let app_name = "ETH ISOLATION";
    let version = env!("CARGO_PKG_VERSION");
    // let datetime = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Draw a border around the whole header area
    let border_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TT_GREEN));

    // Compute the inner area before rendering the widget
    let inner_area = border_block.inner(area);
    frame.render_widget(border_block, area);

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Version
            Constraint::Percentage(60), // App name
            Constraint::Percentage(20), // Datetime
        ])
        .split(inner_area);

    // Create three spans for version, app name, and datetime
    let version_span = Span::styled(format!(" v{version} "), Style::default().fg(Color::Yellow));
    let app_name_span = Span::styled(app_name, Style::default().fg(Color::Yellow).bold());
    let datetime_span = Span::styled(format!(" {current_time} "), Style::default().fg(Color::Yellow));

    // Place each in its own area according to the header_layout
    let version_paragraph = Paragraph::new(Line::from(vec![version_span.clone()]))
        .alignment(Alignment::Left);
    let app_name_paragraph = Paragraph::new(Line::from(vec![app_name_span.clone()]))
        .alignment(Alignment::Center);
    let datetime_paragraph = Paragraph::new(Line::from(vec![datetime_span.clone()]))
        .alignment(Alignment::Right);

    frame.render_widget(version_paragraph, header_layout[0]);
    frame.render_widget(app_name_paragraph, header_layout[1]);
    frame.render_widget(datetime_paragraph, header_layout[2]);

}

fn render_footer(frame: &mut Frame, area: Rect) {
    // Draw a border around the whole footer area
    let border_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TT_GREEN));

    // Compute the inner area before rendering the widget
    let inner_area = border_block.inner(area);
    frame.render_widget(border_block, area);

    let footer_text = Paragraph::new("Use ↑↓←→ to scroll, PgUp/PgDown for faster scrolling, 'q' to quit")
        .style(Style::default().fg(Color::Yellow).bold())
        .alignment(Alignment::Center);
    frame.render_widget(footer_text, inner_area);
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();
    while !app.should_quit {
        terminal.draw(|f| ui_layout(f, app))?;

        // Handle events and timing
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

    }
    Ok(())
}

fn ui_layout(f: &mut Frame, app: &mut App) {
    // Main layout with header and body
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Narrow header
            Constraint::Min(0),    // Remaining space for columns
            Constraint::Length(3), // Help text at the bottom
        ])
        .split(f.area());

    // Header
    let curr_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    render_header(f, main_chunks[0], curr_time);

    // Footer - help text at the bottom
    render_footer(f, main_chunks[2]);

    // Split the remaining space into two columns
    let column_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(35), // First column - 35 pixels
            Constraint::Length(80), // Second column - 80 pixels
            Constraint::Min(0),     // Remaining space (for scrollbars)
        ])
        .split(main_chunks[1]);

    // Create content for both columns (simulated long content)
    let col1_content: Vec<Line> = (0..app.content_height)
        .map(|i| Line::from(vec![
            Span::raw(format!("COL 1 - Line {}", i + 1))
        ]))
        .collect();

    let col2_content: Vec<Line> = (0..app.content_height)
        .map(|i| Line::from(vec![
            Span::raw(format!("COL 2 - Line {} with some additional text to demonstrate horizontal scrolling", i + 1))
        ]))
        .collect();

    // Column 1 with scrollbar
    let col1_block = Block::default()
        .borders(Borders::ALL)
        .title("Column 1");

    let col1_paragraph = Paragraph::new(col1_content.clone())
        .block(col1_block)
        .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

    f.render_widget(col1_paragraph, column_chunks[0]);

    // Vertical scrollbar for column 1
    let col1_vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    f.render_stateful_widget(
        col1_vertical_scrollbar,
        column_chunks[0],
        &mut app.vertical_scroll_state,
    );

    // Column 2 with scrollbar
    let col2_block = Block::default()
        .borders(Borders::ALL)
        .title("Column 2");

    let col2_paragraph = Paragraph::new(col2_content.clone())
        .block(col2_block)
        .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

    f.render_widget(col2_paragraph, column_chunks[1]);

    // Vertical scrollbar for column 2
    let col2_vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    f.render_stateful_widget(
        col2_vertical_scrollbar,
        column_chunks[1],
        &mut app.vertical_scroll_state,
    );

    // Horizontal scrollbar (shared for both columns)
    let horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
        .begin_symbol(Some("←"))
        .end_symbol(Some("→"));

    // Create area for horizontal scrollbar that spans both columns
    let horizontal_scrollbar_area = Rect {
        x: column_chunks[0].x,
        y: column_chunks[0].y + column_chunks[0].height - 1,
        width: column_chunks[1].x + column_chunks[1].width - column_chunks[0].x,
        height: 1,
    };

    f.render_stateful_widget(
        horizontal_scrollbar,
        horizontal_scrollbar_area,
        &mut app.horizontal_scroll_state,
    );


}
