use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table, TableState},
    Frame, Terminal,
};

use crate::cpu::class::CPUState;

struct App {
    cpu_state: CPUState,
    table_state: TableState,
}

impl App {
    fn new() -> Self {
        let cpu_state = CPUState::new();
        let mut table_state = TableState::default();

        if !cpu_state.cpu.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            cpu_state,
            table_state,
        }
    }

    fn select_next(&mut self) {
        if self.cpu_state.cpu.is_empty() {
            self.table_state.select(None);
            return;
        }

        let last = self.cpu_state.cpu.len() - 1;
        let index = self.table_state.selected().unwrap_or(0);
        let next = if index >= last { 0 } else { index + 1 };
        self.table_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.cpu_state.cpu.is_empty() {
            self.table_state.select(None);
            return;
        }

        let last = self.cpu_state.cpu.len() - 1;
        let index = self.table_state.selected().unwrap_or(0);
        let previous = if index == 0 { last } else { index - 1 };
        self.table_state.select(Some(previous));
    }

    fn refresh(&mut self) {
        let selected = self.table_state.selected().unwrap_or(0);
        self.cpu_state = CPUState::new();

        if self.cpu_state.cpu.is_empty() {
            self.table_state.select(None);
            return;
        }

        let clamped = selected.min(self.cpu_state.cpu.len().saturating_sub(1));
        self.table_state.select(Some(clamped));
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();
    let tick_rate = Duration::from_millis(1000);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
                        KeyCode::Char('r') => app.refresh(),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.refresh();
            last_tick = Instant::now();
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let header = Row::new(vec![
        Cell::from("cpu id"),
        Cell::from("path"),
        Cell::from("governor"),
        Cell::from("cur kHz"),
        Cell::from("min"),
        Cell::from("max"),
        Cell::from("driver"),
    ])
    .style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let rows = app.cpu_state.cpu.iter().map(|cpu| {
        Row::new(vec![
            Cell::from(cpu.id.to_string()),
            Cell::from(cpu.path.display().to_string()),
            Cell::from(cpu.scaling_governor.clone()),
            Cell::from(cpu.scaling_cur_freq.clone()),
            Cell::from(cpu.scaling_min_freq.clone()),
            Cell::from(cpu.scaling_max_freq.clone()),
            Cell::from(cpu.scaling_driver.clone()),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Percentage(35),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
    .highlight_symbol("> ");

    frame.render_stateful_widget(table, frame.area(), &mut app.table_state);
}