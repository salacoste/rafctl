use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, TableState};
use ratatui::{DefaultTerminal, Frame};

use crate::core::profile::{list_profiles, load_profile, AuthMode, ToolType};
use crate::error::RafctlError;
use crate::tools::is_authenticated;

#[cfg(target_os = "macos")]
use crate::cli::quota::UsageLimits;

struct ProfileRow {
    name: String,
    tool: ToolType,
    auth_mode: AuthMode,
    authenticated: bool,
    last_used: Option<String>,
    #[cfg(target_os = "macos")]
    #[allow(dead_code)]
    usage: Option<UsageLimits>,
}

struct App {
    profiles: Vec<ProfileRow>,
    table_state: TableState,
    should_quit: bool,
    message: Option<String>,
}

impl App {
    fn new() -> Result<Self, RafctlError> {
        let profile_names = list_profiles()?;
        let mut profiles = Vec::new();

        for name in profile_names {
            if let Ok(profile) = load_profile(&name) {
                let authenticated = is_authenticated(profile.tool, &name).unwrap_or(false);
                let last_used = profile
                    .last_used
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string());

                profiles.push(ProfileRow {
                    name: profile.name,
                    tool: profile.tool,
                    auth_mode: profile.auth_mode,
                    authenticated,
                    last_used,
                    #[cfg(target_os = "macos")]
                    usage: None,
                });
            }
        }

        let mut table_state = TableState::default();
        if !profiles.is_empty() {
            table_state.select(Some(0));
        }

        Ok(Self {
            profiles,
            table_state,
            should_quit: false,
            message: None,
        })
    }

    fn next(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + 1) % self.profiles.len(),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.profiles.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn selected_profile(&self) -> Option<&ProfileRow> {
        self.table_state
            .selected()
            .and_then(|i| self.profiles.get(i))
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Down | KeyCode::Char('j') => self.next(),
                KeyCode::Up | KeyCode::Char('k') => self.previous(),
                KeyCode::Enter | KeyCode::Char('r') => {
                    if let Some(profile) = self.selected_profile() {
                        self.message = Some(format!(
                            "Would run: rafctl run {} (press 'q' to exit first)",
                            profile.name
                        ));
                    }
                }
                KeyCode::Char('l') => {
                    if let Some(profile) = self.selected_profile() {
                        self.message = Some(format!(
                            "Would login: rafctl auth login {} (press 'q' to exit first)",
                            profile.name
                        ));
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn run_dashboard() -> Result<(), RafctlError> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut DefaultTerminal) -> Result<(), RafctlError> {
    let mut app = App::new()?;

    loop {
        terminal
            .draw(|frame| render(frame, &mut app))
            .map_err(|e| RafctlError::ConfigWrite {
                path: std::path::PathBuf::from("terminal"),
                source: io::Error::other(e),
            })?;

        if event::poll(Duration::from_millis(100)).map_err(|e| RafctlError::ConfigRead {
            path: std::path::PathBuf::from("events"),
            source: io::Error::other(e),
        })? {
            let event = event::read().map_err(|e| RafctlError::ConfigRead {
                path: std::path::PathBuf::from("events"),
                source: io::Error::other(e),
            })?;
            app.handle_event(event);
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, app: &mut App) {
    let [header_area, table_area, help_area, message_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header_area);
    render_table(frame, app, table_area);
    render_help(frame, help_area);
    render_message(frame, app, message_area);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "rafctl ",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("dashboard"),
    ]))
    .block(Block::bordered().title("AI Coding Agent Profile Manager ☕"));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let header = Row::new(vec!["Name", "Tool", "Auth Mode", "Status", "Last Used"])
        .style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .profiles
        .iter()
        .map(|p| {
            let status = if p.authenticated {
                Cell::from("✓ Auth").style(Style::new().fg(Color::Green))
            } else {
                Cell::from("✗ No Auth").style(Style::new().fg(Color::Red))
            };

            let auth_mode = match p.auth_mode {
                AuthMode::OAuth => "oauth",
                AuthMode::ApiKey => "api-key",
            };

            Row::new(vec![
                Cell::from(p.name.clone()),
                Cell::from(p.tool.to_string()),
                Cell::from(auth_mode),
                status,
                Cell::from(p.last_used.clone().unwrap_or_else(|| "never".to_string())),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(35),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::bordered().title("Profiles"))
        .column_spacing(1)
        .style(Style::new().fg(Color::White))
        .row_highlight_style(
            Style::new()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_help(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑/k", Style::new().fg(Color::Cyan)),
        Span::raw(" up  "),
        Span::styled("↓/j", Style::new().fg(Color::Cyan)),
        Span::raw(" down  "),
        Span::styled("Enter/r", Style::new().fg(Color::Cyan)),
        Span::raw(" run  "),
        Span::styled("l", Style::new().fg(Color::Cyan)),
        Span::raw(" login  "),
        Span::styled("q/Esc", Style::new().fg(Color::Cyan)),
        Span::raw(" quit"),
    ]))
    .block(Block::bordered());

    frame.render_widget(help, area);
}

fn render_message(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if let Some(msg) = &app.message {
        let message = Paragraph::new(msg.as_str()).style(Style::new().fg(Color::Yellow));
        frame.render_widget(message, area);
    }
}
