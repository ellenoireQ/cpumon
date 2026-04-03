use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
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
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use crate::cpu::class::CPUState;

struct App {
    cpu_state: CPUState,
    table_state: TableState,
    process_tree_lines: Vec<String>,
}

impl App {
    fn new() -> Self {
        let cpu_state = CPUState::new();
        let mut table_state = TableState::default();

        if !cpu_state.cpu.is_empty() {
            table_state.select(Some(0));
        }

        let mut app = Self {
            cpu_state,
            table_state,
            process_tree_lines: Vec::new(),
        };

        app.update_process_tree();
        app
    }

    fn select_next(&mut self) {
        if self.cpu_state.cpu.is_empty() {
            self.table_state.select(None);
            self.update_process_tree();
            return;
        }

        let last = self.cpu_state.cpu.len() - 1;
        let index = self.table_state.selected().unwrap_or(0);
        let next = if index >= last { 0 } else { index + 1 };
        self.table_state.select(Some(next));
        self.update_process_tree();
    }

    fn select_previous(&mut self) {
        if self.cpu_state.cpu.is_empty() {
            self.table_state.select(None);
            self.update_process_tree();
            return;
        }

        let last = self.cpu_state.cpu.len() - 1;
        let index = self.table_state.selected().unwrap_or(0);
        let previous = if index == 0 { last } else { index - 1 };
        self.table_state.select(Some(previous));
        self.update_process_tree();
    }

    fn refresh(&mut self) {
        let selected = self.table_state.selected().unwrap_or(0);
        self.cpu_state = CPUState::new();

        if self.cpu_state.cpu.is_empty() {
            self.table_state.select(None);
            self.update_process_tree();
            return;
        }

        let clamped = selected.min(self.cpu_state.cpu.len().saturating_sub(1));
        self.table_state.select(Some(clamped));
        self.update_process_tree();
    }

    fn selected_cpu_id(&self) -> Option<u32> {
        self.table_state
            .selected()
            .and_then(|index| self.cpu_state.cpu.get(index))
            .map(|cpu| cpu.id)
    }

    fn update_process_tree(&mut self) {
        self.process_tree_lines = match self.selected_cpu_id() {
            Some(cpu_id) => build_process_tree_lines(cpu_id),
            None => vec![String::from("no cpu selected")],
        };
    }
}

#[derive(Clone)]
struct ProcessInfo {
    pid: i32,
    ppid: i32,
    name: String,
    last_cpu: Option<u32>,
}

fn build_process_tree_lines(cpu_id: u32) -> Vec<String> {
    let process_map = read_processes();
    if process_map.is_empty() {
        return vec![
            format!("process tree cpu {}", cpu_id),
            String::from("no process data from /proc"),
        ];
    }

    let involved: Vec<i32> = process_map
        .values()
        .filter(|proc| proc.last_cpu == Some(cpu_id))
        .map(|proc| proc.pid)
        .collect();

    if involved.is_empty() {
        return vec![
            format!("process tree cpu {}", cpu_id),
            String::from("no process currently scheduled on this cpu"),
        ];
    }

    let mut included: HashSet<i32> = involved.iter().copied().collect();
    for pid in &involved {
        let mut current = *pid;
        let mut guard = 0;

        while let Some(proc) = process_map.get(&current) {
            let parent = proc.ppid;
            if parent <= 0 || parent == current {
                break;
            }

            included.insert(parent);
            current = parent;
            guard += 1;

            if guard > 128 {
                break;
            }
        }
    }

    let mut children: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    let mut roots: Vec<i32> = Vec::new();

    for pid in &included {
        if let Some(proc) = process_map.get(pid) {
            if included.contains(&proc.ppid) && proc.ppid != proc.pid {
                children.entry(proc.ppid).or_default().push(proc.pid);
            } else {
                roots.push(proc.pid);
            }
        }
    }

    roots.sort_unstable();
    roots.dedup();
    for child_list in children.values_mut() {
        child_list.sort_unstable();
    }

    let mut lines = vec![
        format!("process tree cpu {}", cpu_id),
        String::from("* marker = currently on selected cpu"),
    ];

    for (index, root) in roots.iter().enumerate() {
        let is_last_root = index + 1 == roots.len();
        append_tree_lines(
            *root,
            "",
            is_last_root,
            true,
            cpu_id,
            &process_map,
            &children,
            &mut lines,
        );
    }

    lines
}

fn append_tree_lines(
    pid: i32,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    cpu_id: u32,
    process_map: &HashMap<i32, ProcessInfo>,
    children: &BTreeMap<i32, Vec<i32>>,
    lines: &mut Vec<String>,
) {
    let Some(proc) = process_map.get(&pid) else {
        return;
    };

    let marker = if proc.last_cpu == Some(cpu_id) { " *" } else { "" };
    let line = if is_root {
        format!("{} {}{}", proc.pid, proc.name, marker)
    } else {
        let branch = if is_last { "`-- " } else { "|-- " };
        format!("{}{}{} {}{}", prefix, branch, proc.pid, proc.name, marker)
    };
    lines.push(line);

    let Some(child_pids) = children.get(&pid) else {
        return;
    };

    let next_prefix = if is_root {
        String::new()
    } else {
        format!("{}{}", prefix, if is_last { "    " } else { "|   " })
    };

    for (index, child_pid) in child_pids.iter().enumerate() {
        let child_is_last = index + 1 == child_pids.len();
        append_tree_lines(
            *child_pid,
            &next_prefix,
            child_is_last,
            false,
            cpu_id,
            process_map,
            children,
            lines,
        );
    }
}

fn read_processes() -> HashMap<i32, ProcessInfo> {
    let mut map = HashMap::new();

    let Ok(entries) = fs::read_dir("/proc") else {
        return map;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(pid_str) = file_name.to_str() else {
            continue;
        };

        let Ok(pid) = pid_str.parse::<i32>() else {
            continue;
        };

        let stat_path = format!("/proc/{}/stat", pid);
        let Ok(stat_content) = fs::read_to_string(stat_path) else {
            continue;
        };

        let Some((name, ppid, last_cpu)) = parse_stat(&stat_content) else {
            continue;
        };

        map.insert(
            pid,
            ProcessInfo {
                pid,
                ppid,
                name,
                last_cpu,
            },
        );
    }

    map
}

fn parse_stat(content: &str) -> Option<(String, i32, Option<u32>)> {
    let open_paren = content.find('(')?;
    let close_paren = content.rfind(')')?;
    if close_paren + 2 > content.len() {
        return None;
    }

    let name = content[(open_paren + 1)..close_paren].to_string();
    let rest = &content[(close_paren + 2)..];
    let fields: Vec<&str> = rest.split_whitespace().collect();
    if fields.len() <= 36 {
        return None;
    }

    let ppid = fields.get(1)?.parse::<i32>().ok()?;
    let last_cpu = fields.get(36).and_then(|value| value.parse::<u32>().ok());

    Some((name, ppid, last_cpu))
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
    let area = frame.area();
    let table_rows = app.cpu_state.cpu.len() as u16;
    let desired_table_height = table_rows.saturating_add(1);
    let max_table_height = area.height.saturating_sub(2);
    let table_height = desired_table_height.clamp(1, max_table_height.max(1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(table_height),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);

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

    frame.render_stateful_widget(table, chunks[0], &mut app.table_state);

    let process_lines = app
        .process_tree_lines
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect::<Vec<_>>();
    let process_tree = Paragraph::new(process_lines);

    frame.render_widget(process_tree, chunks[2]);
}