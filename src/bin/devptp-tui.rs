use std::{
    collections::VecDeque,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use devptp::{Daemon, Role, Status};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline, Wrap},
};

const LOG_LIMIT: usize = 200;

struct App {
    daemon: std::sync::Arc<Daemon>,
    ticket: String,
    editing: bool,
    confirm_quit: bool,
    status: Option<Status>,
    rtt: Option<Duration>,
    started: Instant,
    last_ping: Instant,
    last_traffic: Instant,
    traffic_samples: VecDeque<(u64, u64)>,
    logs: VecDeque<String>,
}

impl App {
    fn new(daemon: std::sync::Arc<Daemon>) -> Self {
        let now = Instant::now();
        let mut app = Self {
            daemon,
            ticket: String::new(),
            editing: true,
            confirm_quit: false,
            status: None,
            rtt: None,
            started: now,
            last_ping: now,
            last_traffic: now,
            traffic_samples: VecDeque::new(),
            logs: VecDeque::new(),
        };
        app.log("devptp client ready");
        app.log("Paste a serving ticket and press Enter to connect");
        app
    }

    fn log(&mut self, message: impl Into<String>) {
        if self.logs.len() == LOG_LIMIT {
            self.logs.pop_front();
        }
        self.logs.push_back(message.into());
    }

    fn connected(&self) -> bool {
        self.status.as_ref().is_some_and(|status| status.connected)
    }

    async fn refresh(&mut self) {
        self.status = Some(self.daemon.status().await);
        if self.last_traffic.elapsed() >= Duration::from_secs(1) {
            let traffic = self.daemon.traffic();
            self.traffic_samples
                .push_back((traffic.sent, traffic.received));
            while self.traffic_samples.len() > 120 {
                self.traffic_samples.pop_front();
            }
            self.last_traffic = Instant::now();
        }
        if self.connected() && self.last_ping.elapsed() >= Duration::from_secs(5) {
            self.last_ping = Instant::now();
            let started = Instant::now();
            match self.daemon.ping().await {
                Ok(()) => {
                    self.rtt = Some(started.elapsed());
                    self.log(format!("ping ok · {} ms", started.elapsed().as_millis()));
                }
                Err(error) => self.log(format!("ping failed · {error}")),
            }
        }
    }

    async fn connect(&mut self) {
        if self.ticket.trim().is_empty() {
            self.log("ticket is empty");
            return;
        }
        self.log("connecting…");
        match self.daemon.start_peer(self.ticket.clone()).await {
            Ok(()) => self.log("connection established"),
            Err(error) => self.log(format!("connection failed · {error}")),
        }
        self.editing = false;
        self.refresh().await;
    }

    async fn reconnect(&mut self) {
        if self.connected() {
            let _ = self.daemon.disconnect().await;
            self.log("disconnected");
        }
        self.editing = true;
        self.log("reconnect with the current ticket, or edit it first");
    }

    async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if self.confirm_quit {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => return true,
                KeyCode::Char('n') | KeyCode::Esc => self.confirm_quit = false,
                _ => {}
            }
            return false;
        }

        if self.editing {
            match key.code {
                KeyCode::Enter => self.connect().await,
                KeyCode::Esc => self.editing = false,
                KeyCode::Backspace => {
                    self.ticket.pop();
                }
                KeyCode::Char(value) => self.ticket.push(value),
                _ => {}
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') => {
                if self.connected() {
                    self.confirm_quit = true;
                    self.log("quit? press y to shut down, n to continue");
                } else {
                    return true;
                }
            }
            KeyCode::Char('c') | KeyCode::Char('e') => self.editing = true,
            KeyCode::Char('d') => {
                match self.daemon.disconnect().await {
                    Ok(()) => self.log("disconnected"),
                    Err(error) => self.log(format!("disconnect failed · {error}")),
                }
                self.refresh().await;
            }
            KeyCode::Char('r') => self.reconnect().await,
            KeyCode::Char('p') => {
                self.last_ping = Instant::now() - Duration::from_secs(5);
                self.refresh().await;
            }
            _ => {}
        }
        false
    }
}

fn draw(frame: &mut Frame<'_>, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(8),
            Constraint::Length(8),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let logo = Paragraph::new(vec![
        Line::from(Span::styled(
            "  DDDD  EEEEE V   V PPPP  TTTTT PPPP ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  D   D E     V   V P   P   T   P   P",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  D   D EEEE   V V  PPPP    T   PPPP ",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  D   D E       V   P       T   P    ",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  DDDD  EEEEE   V   P       T   P    ",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" devptp · client "),
    );
    frame.render_widget(logo, outer[0]);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(outer[1]);
    draw_status(frame, app, middle[0]);
    draw_ports(frame, app, middle[1]);

    let logs: Vec<ListItem<'_>> = app
        .logs
        .iter()
        .rev()
        .map(|log| ListItem::new(log.as_str()))
        .collect();
    frame.render_widget(
        List::new(logs).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" logs · newest first "),
        ),
        outer[2],
    );
    frame.render_widget(
        Paragraph::new(
            " c/e ticket  ·  Enter connect  ·  d disconnect  ·  r reconnect  ·  p ping  ·  q quit ",
        )
        .style(Style::default().fg(Color::Gray)),
        outer[3],
    );

    if app.editing {
        let area = centered_rect(80, 25, frame.area());
        let input = Paragraph::new(app.ticket.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" serving ticket · Enter connect · Esc cancel "),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(input, area);
        frame.set_cursor_position((area.x + 1 + app.ticket.len() as u16, area.y + 1));
    } else if app.confirm_quit {
        let area = centered_rect(60, 20, frame.area());
        let prompt = Paragraph::new("Shut down the daemon and quit?  [y]es / [n]o")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" confirm "));
        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(prompt, area);
    }
}

fn draw_status(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let (state, color) = match app.status.as_ref().map(|status| status.role) {
        Some(Role::Connecting) => ("CONNECTED", Color::Green),
        Some(Role::Serving) => ("SERVING", Color::Yellow),
        _ => ("DISCONNECTED", Color::DarkGray),
    };
    let rtt = app
        .rtt
        .map_or_else(|| "—".into(), |value| format!("{} ms", value.as_millis()));
    let traffic = app.daemon.traffic();
    let text = vec![
        Line::from(vec![
            Span::raw("state   "),
            Span::styled(
                state,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(format!("uptime  {}s", app.started.elapsed().as_secs())),
        Line::from(format!("rtt     {rtt}")),
        Line::from(format!(
            "bytes   ↑ {} · ↓ {}",
            format_bytes(traffic.sent),
            format_bytes(traffic.received)
        )),
        Line::from("traffic application/tunnel"),
    ];
    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(3)])
        .split(area);
    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" connection ")),
        parts[0],
    );
    let traffic: Vec<u64> = app
        .traffic_samples
        .iter()
        .map(|(sent, received)| sent.saturating_add(*received))
        .collect();
    frame.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" bytes · 2 min · sent + received "),
            )
            .data(&traffic)
            .style(Style::default().fg(Color::Green)),
        parts[1],
    );
}

fn draw_ports(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let forwarded = app
        .status
        .as_ref()
        .map_or_else(Vec::new, |status| status.forwarded_ports.clone());
    let available = app
        .status
        .as_ref()
        .map_or_else(Vec::new, |status| status.available_ports.clone());
    let forwarded_text = if forwarded.is_empty() {
        "none".to_string()
    } else {
        forwarded
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let available_text = if available.is_empty() {
        "none".to_string()
    } else {
        available
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    frame.render_widget(
        Paragraph::new(format!(
            "{forwarded_text}\n\nAutomatically exposed as they become available."
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" forwarded local ports "),
        ),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(format!(
            "{available_text}\n\nInformational · the peer controls availability."
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" serving peer ports "),
        ),
        chunks[1],
    );
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} B")
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let daemon = Daemon::start().await?;
    let mut app = App::new(daemon.clone());
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_ui(&mut terminal, &mut app).await;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    daemon.shutdown().await;
    result
}

async fn run_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && app.handle_key(key).await
        {
            break;
        }
        app.refresh().await;
    }
    Ok(())
}
