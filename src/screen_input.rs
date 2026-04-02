use std::path::PathBuf;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Screen};

/// Screen 2: Input Files configuration.
/// Fields: project name, venue, 4 file slots, start button.
pub struct InputFilesState {
    pub selected: usize,      // 0=name, 1=venue, 2..5=file slots, 6=start
    pub editing: bool,        // true when typing into a text field
    pub edit_buf: String,     // current text being edited
    pub cursor_pos: usize,    // cursor position in edit_buf
    pub browsing: bool,       // true when file browser is open
    pub browser: FileBrowser, // file browser state
}

const FIELD_NAME: usize = 0;
const FIELD_VENUE: usize = 1;
const FIELD_SLOT_FIRST: usize = 2;
const FIELD_SLOT_LAST: usize = 5;
const FIELD_START: usize = 6;
const TOTAL_FIELDS: usize = 7;

impl InputFilesState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            editing: false,
            edit_buf: String::new(),
            cursor_pos: 0,
            browsing: false,
            browser: FileBrowser::new(),
        }
    }

    pub fn move_up(&mut self) {
        if self.browsing {
            self.browser.move_up();
        } else if !self.editing && self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.browsing {
            self.browser.move_down();
        } else if !self.editing && self.selected + 1 < TOTAL_FIELDS {
            self.selected += 1;
        }
    }

    pub fn enter(&mut self, app: &mut App) {
        if self.browsing {
            // Select file from browser
            if let Some(path) = self.browser.selected_path() {
                if path.is_dir() {
                    self.browser.enter_dir(&path);
                } else {
                    // Add file to the active slot
                    let slot_idx = self.selected - FIELD_SLOT_FIRST;
                    if let Some(slot) = app.config.slots.get_mut(slot_idx) {
                        if !slot.files.contains(&path) {
                            slot.files.push(path);
                        }
                    }
                    self.browsing = false;
                }
            }
            return;
        }

        if self.editing {
            // Commit edit
            match self.selected {
                FIELD_NAME => app.config.name = self.edit_buf.clone(),
                FIELD_VENUE => app.config.venue = self.edit_buf.clone(),
                _ => {}
            }
            self.editing = false;
            return;
        }

        match self.selected {
            FIELD_NAME => {
                self.editing = true;
                self.edit_buf = app.config.name.clone();
                self.cursor_pos = self.edit_buf.len();
            }
            FIELD_VENUE => {
                self.editing = true;
                self.edit_buf = app.config.venue.clone();
                self.cursor_pos = self.edit_buf.len();
            }
            FIELD_SLOT_FIRST..=FIELD_SLOT_LAST => {
                // Open file browser
                self.browsing = true;
                self.browser = FileBrowser::new();
            }
            FIELD_START => {
                // Validate and start pipeline
                if !app.config.name.is_empty() {
                    app.screen = Screen::PipelineExecution;
                }
            }
            _ => {}
        }
    }

    pub fn type_char(&mut self, ch: char) {
        if self.editing {
            self.edit_buf.insert(self.cursor_pos, ch);
            self.cursor_pos += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.editing && self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.edit_buf.remove(self.cursor_pos);
        }
    }

    pub fn escape(&mut self, app: &mut App) {
        if self.browsing {
            self.browsing = false;
        } else if self.editing {
            self.editing = false;
        } else {
            app.screen = Screen::ProjectSelect;
        }
    }

    pub fn delete_file(&mut self, app: &mut App) {
        // 'd' key removes last file from current slot
        if self.selected >= FIELD_SLOT_FIRST && self.selected <= FIELD_SLOT_LAST && !self.editing {
            let slot_idx = self.selected - FIELD_SLOT_FIRST;
            if let Some(slot) = app.config.slots.get_mut(slot_idx) {
                slot.files.pop();
            }
        }
    }
}

/// Minimal file browser for selecting input files.
pub struct FileBrowser {
    pub dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl std::fmt::Debug for FileBrowser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileBrowser")
            .field("dir", &self.dir)
            .field("entries", &self.entries.len())
            .field("selected", &self.selected)
            .finish()
    }
}

impl FileBrowser {
    pub fn new() -> Self {
        let dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut fb = Self {
            dir: dir.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
        };
        fb.scan_dir(&dir);
        fb
    }

    fn scan_dir(&mut self, dir: &PathBuf) {
        self.entries.clear();
        self.selected = 0;
        self.scroll_offset = 0;

        // Parent directory entry
        if let Some(parent) = dir.parent() {
            self.entries.push(parent.to_path_buf());
        }

        // Read directory contents
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            for entry in rd.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else {
                    files.push(path);
                }
            }
            dirs.sort();
            files.sort();
            self.entries.extend(dirs);
            self.entries.extend(files);
        }
    }

    pub fn enter_dir(&mut self, path: &PathBuf) {
        self.dir = path.clone();
        self.scan_dir(&path.clone());
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.entries.get(self.selected).cloned()
    }
}

pub fn render(area: Rect, buf: &mut Buffer, state: &InputFilesState, app: &App) {
    let block = Block::default()
        .title(" Research Pipeline -- Input Files ")
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

    // If file browser is open, render it as overlay on the right half
    if state.browsing {
        let left_w = inner.width / 2;
        let left_area = Rect::new(inner.left(), inner.top(), left_w, inner.height);
        let right_area = Rect::new(
            inner.left() + left_w,
            inner.top(),
            inner.width - left_w,
            inner.height,
        );
        render_fields(left_area, buf, state, app);
        render_browser(right_area, buf, &state.browser);
        return;
    }

    render_fields(inner, buf, state, app);
}

fn render_fields(area: Rect, buf: &mut Buffer, state: &InputFilesState, app: &App) {
    let tw = area.width as usize;
    let mut y = area.top() + 1;
    let x = area.left() + 2;
    let field_w = tw.saturating_sub(6).min(60);

    // Project name
    {
        let is_sel = state.selected == FIELD_NAME;
        let marker = if is_sel { ">>" } else { "  " };
        let label = format!("{} Project name: ", marker);
        let value = if state.editing && is_sel {
            format!("[{}|]", &state.edit_buf)
        } else if app.config.name.is_empty() {
            "(enter name)".to_string()
        } else {
            app.config.name.clone()
        };

        buf.set_string(x, y, &label, style_label(is_sel));
        buf.set_string(
            x + label.len() as u16,
            y,
            &value[..value.len().min(field_w)],
            style_value(is_sel, state.editing && is_sel),
        );
        y += 2;
    }

    // Venue
    {
        let is_sel = state.selected == FIELD_VENUE;
        let marker = if is_sel { ">>" } else { "  " };
        let label = format!("{} Target venue: ", marker);
        let value = if state.editing && is_sel {
            format!("[{}|]", &state.edit_buf)
        } else if app.config.venue.is_empty() {
            "(e.g. IEEE SMC 2026)".to_string()
        } else {
            app.config.venue.clone()
        };

        buf.set_string(x, y, &label, style_label(is_sel));
        buf.set_string(
            x + label.len() as u16,
            y,
            &value[..value.len().min(field_w)],
            style_value(is_sel, state.editing && is_sel),
        );
        y += 2;
    }

    // Separator
    let sep = "─".repeat(tw.saturating_sub(4).min(80));
    buf.set_string(x, y, &sep, Style::default().fg(Color::DarkGray));
    y += 1;

    // 4 file slots
    for (i, slot) in app.config.slots.iter().enumerate() {
        if y + 3 >= area.bottom() {
            break;
        }
        let field_idx = FIELD_SLOT_FIRST + i;
        let is_sel = state.selected == field_idx;
        let marker = if is_sel { ">>" } else { "  " };
        let req = if slot.required { "*" } else { " " };

        let file_count = slot.files.len();
        let status = if file_count > 0 {
            format!("{} file{} loaded", file_count, if file_count > 1 { "s" } else { "" })
        } else {
            "empty".to_string()
        };

        let line = format!(
            "{}{} {}. {:<24} [Browse...]  {}",
            marker,
            req,
            i + 1,
            slot.label,
            status,
        );
        let len = line.len().min(tw.saturating_sub(2));

        buf.set_string(x, y, &line[..len], style_label(is_sel));
        y += 1;

        // Show description
        let desc_line = format!("       {}", slot.description);
        let desc_len = desc_line.len().min(tw.saturating_sub(2));
        buf.set_string(
            x,
            y,
            &desc_line[..desc_len],
            Style::default().fg(Color::DarkGray),
        );
        y += 1;

        // Show loaded files
        for f in &slot.files {
            if y >= area.bottom().saturating_sub(3) {
                break;
            }
            let fname = f
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| f.to_string_lossy().to_string());
            let file_line = format!("         > {}", fname);
            let fl = file_line.len().min(tw.saturating_sub(2));
            buf.set_string(
                x,
                y,
                &file_line[..fl],
                Style::default().fg(Color::Green),
            );
            y += 1;
        }
        y += 1;
    }

    // Start button
    if y + 2 < area.bottom() {
        let is_sel = state.selected == FIELD_START;
        let can_start = !app.config.name.is_empty();
        let marker = if is_sel { ">>" } else { "  " };

        let top_line = format!("{}╭──────────────────╮", marker);
        let bot_line = format!("{}╰──────────────────╯", marker);
        let mid = if can_start {
            format!("{}│ START PIPELINE   │", marker)
        } else {
            format!("{}│(enter name first)│", marker)
        };

        let color = if !can_start {
            Color::DarkGray
        } else if is_sel {
            Color::White
        } else {
            Color::Green
        };
        let style = if is_sel && can_start {
            Style::default()
                .fg(color)
                .bg(Color::Rgb(20, 60, 20))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        buf.set_string(x, y, &top_line, style);
        y += 1;
        buf.set_string(x, y, &mid, style);
        y += 1;
        buf.set_string(x, y, &bot_line, style);
    }

    // Footer
    let hint = "Up/Down navigate | Enter edit/browse | d delete file | Esc back";
    let hint_y = area.bottom().saturating_sub(1);
    let hint_x = area.left() + (area.width.saturating_sub(hint.len() as u16)) / 2;
    buf.set_string(
        hint_x,
        hint_y,
        hint,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    );
}

fn render_browser(area: Rect, buf: &mut Buffer, browser: &FileBrowser) {
    let block = Block::default()
        .title(" File Browser ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    block.render(area, buf);

    let tw = inner.width as usize;
    let mut y = inner.top();

    // Current directory
    let dir_str = format!(" {}", browser.dir.to_string_lossy());
    let dir_len = dir_str.len().min(tw);
    buf.set_string(
        inner.left(),
        y,
        &dir_str[..dir_len],
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    y += 1;

    let sep = "─".repeat(tw);
    buf.set_string(inner.left(), y, &sep, Style::default().fg(Color::DarkGray));
    y += 1;

    // Entries
    let visible_h = (inner.bottom() - y).saturating_sub(1) as usize;
    let start = if browser.selected >= visible_h {
        browser.selected - visible_h + 1
    } else {
        0
    };

    for (i, entry) in browser.entries.iter().enumerate().skip(start) {
        if y >= inner.bottom().saturating_sub(1) {
            break;
        }
        let is_sel = i == browser.selected;
        let is_dir = entry.is_dir();
        let is_parent = i == 0 && entry.parent().is_some() && entry != &browser.dir;

        let name = if is_parent {
            "..".to_string()
        } else {
            entry
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.to_string_lossy().to_string())
        };

        let prefix = if is_dir { "/" } else { " " };
        let marker = if is_sel { ">>" } else { "  " };
        let line = format!("{} {}{}", marker, prefix, name);
        let len = line.len().min(tw);

        let style = if is_sel {
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(60, 40, 20))
                .add_modifier(Modifier::BOLD)
        } else if is_dir {
            Style::default()
                .fg(Color::Rgb(100, 200, 255))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        buf.set_string(inner.left(), y, &line[..len], style);
        y += 1;
    }

    // Footer
    let hint = "Enter select | Esc cancel";
    let hint_y = inner.bottom().saturating_sub(1);
    buf.set_string(
        inner.left() + 1,
        hint_y,
        hint,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    );
}

fn style_label(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn style_value(selected: bool, editing: bool) -> Style {
    if editing {
        Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(40, 40, 20))
            .add_modifier(Modifier::BOLD)
    } else if selected {
        Style::default().fg(Color::Rgb(200, 180, 255))
    } else {
        Style::default().fg(Color::Rgb(180, 140, 255))
    }
}
