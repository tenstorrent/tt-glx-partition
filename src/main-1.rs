use color_eyre::Result;
use chrono::Local;
use crossterm::event::{self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
    KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Stylize,Style, Color},
    text::{Line, Text, Span},
    widgets::{Block, Paragraph, BorderType, Borders},
    DefaultTerminal,
    Frame
};
mod ui_theme;
const SPACER_HEIGHT: u16 = 0;
const ILLUSTRATION_HEIGHT: u16 = 4;
const EXAMPLE_HEIGHT: u16 = ILLUSTRATION_HEIGHT + SPACER_HEIGHT;
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

// Custom cell widget that is like a button. It has a label and 3 states: unselected(default), hover, selected
#[derive(Debug, Clone)]
pub struct CellWidget {
    label: String,
    state: CellState,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Unselected,
    Hover,
    Selected,
}
impl CellWidget {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            state: CellState::Unselected,
        }
    }

    pub fn set_state(&mut self, state: CellState) {
        self.state = state;
    }

    pub fn render(&self) -> Paragraph {
        let styled_label = match self.state {
            CellState::Unselected => self.label.clone().white(),
            CellState::Hover => self.label.clone().yellow().bold(),
            CellState::Selected => self.label.clone().green().bold().underlined(),
        };
        Paragraph::new(styled_label)
            .block(
                Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(BorderType::Double)
            )
            .alignment(ratatui::layout::Alignment::Center)
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let app_name = "My Ratatui App";
    let version = "v1.0.0";
    // Store the current time in the App struct and update it every second.
    let datetime = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let line = Line::from(vec![
        Span::styled(format!(" {version} "), Style::default().fg(Color::Blue)),
        Span::raw(""),
        Span::styled(app_name, Style::default().fg(Color::White).bold()),
        Span::raw(""),
        Span::styled(format!(" {datetime} "), Style::default().fg(Color::Green)),
    ]);

    let paragraph = Paragraph::new(line)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray))
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    /// Is the application running?
    running: bool,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    // Main layout of the app
    fn render(frame: &mut Frame){
        // make a layout of 32 chips, 4 columns and 8 rows
        // Use 75% of the given area for the chip grid
        let grid_area = Rect {
            x: frame.area().x,
            y: frame.area().y + (frame.area().height as f32 * 0.25) as u16 / 2,
            width: frame.area().width,
            height: (frame.area().height as f32 * 0.75) as u16,
        };
        let column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            ])
            .split(grid_area);
        // Put the chip name in the center of each chip
        for (i, chunk) in column_layout.iter().enumerate() {

            let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 1),
                Constraint::Ratio(1, 1),
                Constraint::Ratio(1, 1),
                Constraint::Ratio(1, 1),
            ])
            .split(*chunk);
            for (j, row_chunk) in row_layout.iter().enumerate() {
                let chip_name = format!("Chip {}", i + 1 + j * 4);
                let cell_widget = CellWidget::new(&chip_name);
                frame.render_widget(cell_widget.render(), *row_chunk);
            }
        }
        // Render the header with the current time.
        // The header will be updated every second because the main loop redraws the UI on each event.
        render_header(frame, Rect {
            x: frame.area().x,
            y: frame.area().y,
            width: frame.area().width,
            height: 3,
        });
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        self.update_max_scroll_offset();
        while self.running {
            terminal.draw(|frame| Self::render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn update_max_scroll_offset(&mut self) {
        self.max_scroll_offset = EXAMPLE_HEIGHT;
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (KeyModifiers::NONE, KeyCode::Left) => self.decrement(),
            (KeyModifiers::NONE, KeyCode::Right) => self.increment(),
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    // Increment and decrement counter
    fn increment(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
    fn decrement(&mut self) {
        self.counter = self.counter.wrapping_sub(1);
    }

}
