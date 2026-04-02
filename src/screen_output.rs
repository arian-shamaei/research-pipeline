use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Screen};

/// Screen 4: Output -- shows generated files, stats, and next actions.
pub struct OutputState {
    pub selected: usize, // 0..N-1 = output files, N..N+2 = actions
}

const ACTION_REVISE: usize = 100;
const ACTION_NEW: usize = 101;
const ACTION_QUIT: usize = 102;

impl OutputState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn move_up(&mut self) {
        match self.selected {
            0 => {}
            ACTION_REVISE => self.selected = 3, // jump back to last file row
            ACTION_NEW => self.selected = ACTION_REVISE,
            ACTION_QUIT => self.selected = ACTION_NEW,
            n => self.selected = n - 1,
        }
    }

    pub fn move_down(&mut self) {
        match self.selected {
            0..=2 => self.selected += 1,
            3 => self.selected = ACTION_REVISE,
            ACTION_REVISE => self.selected = ACTION_NEW,
            ACTION_NEW => self.selected = ACTION_QUIT,
            _ => {}
        }
    }

    pub fn enter(&self, app: &mut App) {
        match self.selected {
            ACTION_REVISE => {
                // Go back to pipeline to revise
                app.screen = Screen::PipelineExecution;
            }
            ACTION_NEW => {
                app.config = crate::app::ProjectConfig::default();
                app.screen = Screen::ProjectSelect;
            }
            ACTION_QUIT => {
                app.should_quit = true;
            }
            _ => {
                // Could open the file -- for now no-op
            }
        }
    }
}

pub fn render(area: Rect, buf: &mut Buffer, state: &OutputState, app: &App) {
    let block = Block::default()
        .title(format!(
            " Research Pipeline -- Output: {} ",
            app.config.name
        ))
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 40, 100)));
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 8 || inner.width < 40 {
        return;
    }

    let tw = inner.width as usize;
    let mut y = inner.top() + 1;
    let x = inner.left() + 2;

    // Header
    buf.set_string(
        x,
        y,
        "GENERATED FILES",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    );
    y += 2;

    // Output files
    let output_dir = &app.config.output_dir;
    let output_files = [
        (
            "PDF",
            output_dir.join(format!("{}.pdf", app.config.name)),
            "Final compiled paper",
        ),
        (
            "LaTeX",
            output_dir.join(format!("{}.tex", app.config.name)),
            "Editable source document",
        ),
        (
            "Figures",
            output_dir.join("figures"),
            "Generated diagrams and charts",
        ),
        (
            "BibTeX",
            output_dir.join("refs.bib"),
            "Citation database",
        ),
    ];

    for (i, (label, path, desc)) in output_files.iter().enumerate() {
        if y + 1 >= inner.bottom().saturating_sub(8) {
            break;
        }
        let is_sel = state.selected == i;
        let marker = if is_sel { ">>" } else { "  " };
        let exists = path.exists();
        let check = if exists { "[*]" } else { "[ ]" };
        let size = if exists && path.is_file() {
            let meta = std::fs::metadata(path).ok();
            meta.map(|m| {
                let kb = m.len() / 1024;
                if kb > 1024 {
                    format!("{:.1}MB", kb as f64 / 1024.0)
                } else {
                    format!("{}KB", kb)
                }
            })
            .unwrap_or_default()
        } else {
            String::new()
        };

        let fname = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let line = format!(
            "{} {} {:<8} {:<30} {}",
            marker, check, label, fname, size
        );
        let len = line.len().min(tw.saturating_sub(2));

        let style = if is_sel {
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(20, 50, 20))
                .add_modifier(Modifier::BOLD)
        } else if exists {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        buf.set_string(x, y, &line[..len], style);
        y += 1;

        // Description
        let desc_line = format!("         {}", desc);
        buf.set_string(
            x,
            y,
            &desc_line[..desc_line.len().min(tw.saturating_sub(2))],
            Style::default().fg(Color::DarkGray),
        );
        y += 1;
    }

    // Stats section
    y += 1;
    if y + 4 < inner.bottom().saturating_sub(4) {
        let sep = "-".repeat(tw.saturating_sub(4).min(80));
        buf.set_string(x, y, &sep, Style::default().fg(Color::DarkGray));
        y += 1;

        buf.set_string(
            x,
            y,
            "STATS",
            Style::default()
                .fg(Color::Rgb(100, 200, 255))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
        y += 1;

        // Read stats from pipeline JSON if available
        let workspace = std::env::var("OPENCLAW_WORKSPACE")
            .unwrap_or_else(|_| r"C:\Users\Administrator\.openclaw\workspace".to_string());
        let json_path = std::path::Path::new(&workspace).join(".paper_pipeline.json");
        if let Ok(text) = std::fs::read_to_string(&json_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(paper) = val
                    .get("active_papers")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                {
                    if let Some(stats) = paper.get("stats").and_then(|v| v.as_object()) {
                        let mut sx = x + 2;
                        for (key, val) in stats {
                            if y >= inner.bottom().saturating_sub(5) {
                                break;
                            }
                            let val_str = if let Some(s) = val.as_str() {
                                s.to_string()
                            } else if let Some(n) = val.as_f64() {
                                if n == n.floor() {
                                    format!("{:.0}", n)
                                } else {
                                    format!("{:.1}", n)
                                }
                            } else {
                                val.to_string()
                            };
                            let entry = format!("{}:{} ", key, val_str);
                            if sx + entry.len() as u16 > inner.right().saturating_sub(2) {
                                y += 1;
                                sx = x + 2;
                            }
                            buf.set_string(sx, y, key, Style::default().fg(Color::Rgb(140, 100, 200)));
                            sx += key.len() as u16;
                            buf.set_string(sx, y, ":", Style::default().fg(Color::DarkGray));
                            sx += 1;
                            buf.set_string(
                                sx,
                                y,
                                &val_str,
                                Style::default()
                                    .fg(Color::Rgb(200, 180, 255))
                                    .add_modifier(Modifier::BOLD),
                            );
                            sx += val_str.len() as u16 + 2;
                        }
                        y += 2;
                    }
                }
            }
        }
    }

    // Action buttons
    if y + 3 < inner.bottom() {
        let sep = "=".repeat(tw.saturating_sub(4).min(80));
        buf.set_string(
            x,
            y,
            &sep,
            Style::default().fg(Color::Rgb(80, 40, 100)),
        );
        y += 1;

        let actions = [
            (ACTION_REVISE, "REVISE", Color::Yellow),
            (ACTION_NEW, "NEW PROJECT", Color::Rgb(100, 200, 255)),
            (ACTION_QUIT, "QUIT", Color::Red),
        ];

        let mut bx = x + 2;
        for (id, label, color) in &actions {
            let is_sel = state.selected == *id;
            let box_w = label.len() + 4;

            if bx + box_w as u16 + 2 > inner.right() {
                break;
            }

            let top_bot = format!("+{}+", "-".repeat(box_w - 2));
            let mid = format!("| {} |", label);

            let style = if is_sel {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(50, 20, 60))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(*color)
            };
            let border_style = if is_sel {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            buf.set_string(bx, y, &top_bot, border_style);
            buf.set_string(bx, y + 1, &mid, style);
            buf.set_string(bx, y + 2, &top_bot, border_style);
            bx += box_w as u16 + 3;
        }
    }

    // Footer
    let hint = "Up/Down navigate | Enter select | Esc back";
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
