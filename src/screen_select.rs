use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Screen};

/// Screen 1: Project Select
/// [NEW PROJECT]  [OPEN EXISTING]  [RECENT]
pub struct ProjectSelectState {
    pub selected: usize, // 0=New, 1=Open, 2+=Recent items
    pub total_items: usize,
}

impl ProjectSelectState {
    pub fn new(app: &App) -> Self {
        Self {
            selected: 0,
            total_items: 2 + app.recents.len(), // New + Open + recents
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.total_items {
            self.selected += 1;
        }
    }

    pub fn enter(&self, app: &mut App) {
        match self.selected {
            0 => {
                // New Project
                app.config = crate::app::ProjectConfig::default();
                app.screen = Screen::InputFiles;
            }
            1 => {
                // Open existing (for now, same as new -- will add file browser later)
                app.config = crate::app::ProjectConfig::default();
                app.screen = Screen::InputFiles;
            }
            n if n >= 2 => {
                // Open a recent project
                let idx = n - 2;
                if let Some(recent) = app.recents.get(idx) {
                    app.config.name = recent.name.clone();
                    app.config.venue = recent.venue.clone();
                    app.screen = Screen::PipelineExecution;
                }
            }
            _ => {}
        }
    }
}

pub fn render(area: Rect, buf: &mut Buffer, state: &ProjectSelectState, app: &App) {
    let block = Block::default()
        .title(" Research Pipeline -- Project Select ")
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 40, 100)));
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 6 || inner.width < 30 {
        return;
    }

    let tw = inner.width as usize;
    let mut y = inner.top() + 1;
    let x = inner.left() + 2;

    // Header
    let header = "Select a project or create a new one:";
    buf.set_string(
        x,
        y,
        header,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    y += 2;

    // Actions
    let actions = [
        ("NEW PROJECT", "Start a new paper from input files"),
        ("OPEN EXISTING", "Browse for a project directory"),
    ];

    for (i, (label, desc)) in actions.iter().enumerate() {
        let is_sel = state.selected == i;
        let marker = if is_sel { ">>" } else { "  " };

        // Box around label
        let box_w = label.len() + 4;
        let top_bot = format!("{}+{}+", marker, "-".repeat(box_w - 2));
        let mid = format!("{}| {} |", marker, label);

        let sel_style = if is_sel {
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(50, 20, 60))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 140, 255))
        };
        let border_style = if is_sel {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Rgb(80, 40, 100))
        };

        if y + 3 < inner.bottom() {
            buf.set_string(x, y, &top_bot, border_style);
            y += 1;
            buf.set_string(x, y, &mid, sel_style);
            // Description after the box on the same line
            let desc_x = x + mid.len() as u16 + 2;
            if desc_x + desc.len() as u16 <= inner.right() {
                buf.set_string(desc_x, y, desc, Style::default().fg(Color::DarkGray));
            }
            y += 1;
            buf.set_string(x, y, &top_bot, border_style);
            y += 2;
        }
    }

    // Separator
    if y + 1 < inner.bottom() {
        let sep = "-".repeat(tw.saturating_sub(4).min(80));
        buf.set_string(x, y, &sep, Style::default().fg(Color::DarkGray));
        y += 1;
    }

    // Recent projects header
    if y + 1 < inner.bottom() {
        buf.set_string(
            x,
            y,
            "RECENT PROJECTS",
            Style::default()
                .fg(Color::Rgb(100, 200, 255))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
        y += 1;
    }

    if app.recents.is_empty() {
        if y < inner.bottom() {
            buf.set_string(
                x + 2,
                y,
                "(no recent projects)",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            );
        }
    } else {
        for (i, recent) in app.recents.iter().enumerate() {
            if y + 1 >= inner.bottom() {
                break;
            }
            let idx = i + 2; // offset past the two action buttons
            let is_sel = state.selected == idx;
            let marker = if is_sel { ">>" } else { "  " };

            let done_bar_w = 10;
            let filled = (done_bar_w as f64 * recent.stages_done as f64
                / recent.stages_total.max(1) as f64) as usize;
            let bar = format!(
                "[{}{}]",
                "#".repeat(filled),
                ".".repeat(done_bar_w - filled)
            );

            let line = format!(
                "{} {:30} {:18} {}/{} {} {}",
                marker,
                recent.name,
                recent.venue,
                recent.stages_done,
                recent.stages_total,
                bar,
                recent.status,
            );
            let len = line.len().min(tw.saturating_sub(2));

            let style = if is_sel {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(50, 20, 60))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            buf.set_string(x, y, &line[..len], style);

            // Last activity on the right
            if !recent.last_activity.is_empty() {
                let ts = &recent.last_activity;
                let ts_x = inner.right().saturating_sub(ts.len() as u16 + 2);
                if ts_x > x + len as u16 {
                    buf.set_string(ts_x, y, ts, Style::default().fg(Color::DarkGray));
                }
            }
            y += 1;
        }
    }

    // Footer
    let hint = "Up/Down navigate  |  Enter select  |  q quit";
    let hint_y = inner.bottom().saturating_sub(1);
    let hint_x = x + (inner.width.saturating_sub(hint.len() as u16 + 4)) / 2;
    buf.set_string(
        hint_x,
        hint_y,
        hint,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    );
}
