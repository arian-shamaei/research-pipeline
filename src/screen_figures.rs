use std::path::PathBuf;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Screen};

/// Figure proportion rules (from IEEE TIM research):
/// - 6 single-column figures = 1 printed page
/// - 1 display item per 1,000 words
/// - Target: 5-8 figures for a 7-8 page paper
/// - Single-col: 3.5in, Double-col: 7.16in
const FIGURES_PER_PAGE: f64 = 0.75; // ~1 figure per 1.3 pages

/// Priority levels for figure selection algorithm
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FigurePriority {
    Essential,  // System diagram, key result, comparison
    Expected,   // Setup photo, calibration, additional results
    Optional,   // Flowcharts, supplementary, edge cases
}

#[derive(Debug, Clone)]
pub struct FigureEntry {
    pub id: String,
    pub caption: String,
    pub fig_type: String,       // "bar", "line", "scatter", "block", "photo"
    pub priority: FigurePriority,
    pub width: &'static str,    // "single" or "double" column
    pub pdf_path: Option<PathBuf>,
    pub png_path: Option<PathBuf>,
    pub gp_script: Option<PathBuf>,
    pub status: FigureStatus,
    pub iteration: u8,
    pub max_iterations: u8,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FigureStatus {
    Pending,
    Generating,
    Generated,
    Verifying,
    NeedsRevision,
    Accepted,
    Failed,
}

impl FigureStatus {
    fn label(&self) -> &str {
        match self {
            Self::Pending => "PENDING",
            Self::Generating => "GENERATING...",
            Self::Generated => "GENERATED",
            Self::Verifying => "VERIFYING...",
            Self::NeedsRevision => "NEEDS REVISION",
            Self::Accepted => "ACCEPTED",
            Self::Failed => "FAILED",
        }
    }
    fn color(&self) -> Color {
        match self {
            Self::Pending => Color::DarkGray,
            Self::Generating | Self::Verifying => Color::Yellow,
            Self::Generated => Color::Rgb(100, 200, 255),
            Self::NeedsRevision => Color::Rgb(255, 165, 0),
            Self::Accepted => Color::Green,
            Self::Failed => Color::Red,
        }
    }
}

pub struct FiguresState {
    pub figures: Vec<FigureEntry>,
    pub selected: usize,
    pub page_count: u16,
    pub show_detail: bool,
}

impl FiguresState {
    pub fn new(app: &App) -> Self {
        let figures = Self::scan_figures(app);
        Self {
            figures,
            selected: 0,
            page_count: 7,
            show_detail: false,
        }
    }

    /// Scan output/figures/ for existing figures and build the list
    fn scan_figures(app: &App) -> Vec<FigureEntry> {
        let fig_dir = app.config.output_dir.join("figures");
        let mut figures = vec![
            FigureEntry {
                id: "fig1".into(),
                caption: "Hardware cost comparison of existing leak detection systems and the proposed GASLEAD system.".into(),
                fig_type: "bar".into(),
                priority: FigurePriority::Essential,
                width: "single",
                pdf_path: None, png_path: None, gp_script: None,
                status: FigureStatus::Pending,
                iteration: 0, max_iterations: 3,
                issues: vec![],
            },
            FigureEntry {
                id: "fig2".into(),
                caption: "UW IAC compressed air energy savings from leak detection recommendations (2022--2024).".into(),
                fig_type: "bar+line".into(),
                priority: FigurePriority::Essential,
                width: "single",
                pdf_path: None, png_path: None, gp_script: None,
                status: FigureStatus::Pending,
                iteration: 0, max_iterations: 3,
                issues: vec![],
            },
            FigureEntry {
                id: "fig3".into(),
                caption: "Simulated CEM transfer function H(f) for steel and PVC pipe segments.".into(),
                fig_type: "line".into(),
                priority: FigurePriority::Essential,
                width: "single",
                pdf_path: None, png_path: None, gp_script: None,
                status: FigureStatus::Pending,
                iteration: 0, max_iterations: 3,
                issues: vec![],
            },
            FigureEntry {
                id: "fig4".into(),
                caption: "Simulated detection performance (ROC curves) comparing GASLEAD with CEM calibration, without CEM, and fixed-threshold approaches.".into(),
                fig_type: "line".into(),
                priority: FigurePriority::Essential,
                width: "single",
                pdf_path: None, png_path: None, gp_script: None,
                status: FigureStatus::Pending,
                iteration: 0, max_iterations: 3,
                issues: vec![],
            },
            FigureEntry {
                id: "fig5".into(),
                caption: "GASLEAD system architecture showing distributed sensor nodes, CEM, LoRa gateway, and backend infrastructure.".into(),
                fig_type: "block".into(),
                priority: FigurePriority::Essential,
                width: "double",
                pdf_path: None, png_path: None, gp_script: None,
                status: FigureStatus::Pending,
                iteration: 0, max_iterations: 3,
                issues: vec![],
            },
        ];

        // Check which files exist on disk
        for fig in &mut figures {
            let pdf = fig_dir.join(format!("{}_*.pdf", fig.id));
            let png = fig_dir.join(format!("{}_*.png", fig.id));

            // Scan for matching files
            if let Ok(entries) = std::fs::read_dir(&fig_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&fig.id) {
                        if name.ends_with(".pdf") {
                            fig.pdf_path = Some(entry.path());
                            fig.status = FigureStatus::Generated;
                        } else if name.ends_with(".png") {
                            fig.png_path = Some(entry.path());
                        } else if name.ends_with(".gp") {
                            fig.gp_script = Some(entry.path());
                        }
                    }
                }
            }

            // If both PDF and PNG exist, mark as accepted (initial pass)
            if fig.pdf_path.is_some() && fig.png_path.is_some() {
                fig.status = FigureStatus::Accepted;
                fig.iteration = 1;
            }
        }

        figures
    }

    fn recommended_count(&self) -> usize {
        (self.page_count as f64 * FIGURES_PER_PAGE).round() as usize
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.figures.len() {
            self.selected += 1;
        }
    }

    pub fn toggle_detail(&mut self) {
        self.show_detail = !self.show_detail;
    }

    pub fn escape(&mut self, app: &mut App) {
        if self.show_detail {
            self.show_detail = false;
        } else {
            app.screen = Screen::PipelineExecution;
        }
    }
}

pub fn render(area: Rect, buf: &mut Buffer, state: &FiguresState, _app: &App) {
    let block = Block::default()
        .title(" Research Pipeline -- Figures ")
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 40, 100)));
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 6 || inner.width < 40 {
        return;
    }

    let tw = inner.width as usize;
    let mut y = inner.top() + 1;
    let x = inner.left() + 2;

    // Header stats
    let total = state.figures.len();
    let accepted = state.figures.iter().filter(|f| f.status == FigureStatus::Accepted).count();
    let recommended = state.recommended_count();
    let header = format!(
        "Figures: {}/{} accepted | Recommended: {} for {} pages | Rule: 1 per {:.1} pages",
        accepted, total, recommended, state.page_count,
        1.0 / FIGURES_PER_PAGE
    );
    buf.set_string(x, y, &header[..header.len().min(tw - 4)],
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    y += 1;

    // Proportion bar
    let bar_w = tw.saturating_sub(6).min(60);
    let filled = (bar_w as f64 * accepted as f64 / recommended.max(1) as f64).min(bar_w as f64) as usize;
    let bar_color = if accepted >= recommended { Color::Green }
        else if accepted >= recommended / 2 { Color::Yellow }
        else { Color::Red };
    let bar = format!("[{}{}] {}/{}",
        "#".repeat(filled), ".".repeat(bar_w - filled),
        accepted, recommended);
    buf.set_string(x, y, &bar, Style::default().fg(bar_color));
    y += 2;

    // Separator
    let sep = "─".repeat(tw.saturating_sub(4).min(80));
    buf.set_string(x, y, &sep, Style::default().fg(Color::DarkGray));
    y += 1;

    if state.show_detail {
        // Detail view for selected figure
        if let Some(fig) = state.figures.get(state.selected) {
            render_detail(inner, y, buf, fig);
        }
        return;
    }

    // Figure list
    for (i, fig) in state.figures.iter().enumerate() {
        if y + 3 >= inner.bottom() {
            break;
        }

        let is_sel = state.selected == i;
        let marker = if is_sel { ">>" } else { "  " };
        let prio = match fig.priority {
            FigurePriority::Essential => "P1",
            FigurePriority::Expected  => "P2",
            FigurePriority::Optional  => "P3",
        };
        let width_tag = if fig.width == "double" { "2col" } else { "1col" };
        let iter_tag = if fig.iteration > 0 {
            format!("iter {}/{}", fig.iteration, fig.max_iterations)
        } else {
            String::new()
        };

        // Status line
        let status_str = fig.status.label();
        let line = format!(
            "{} {} [{}] [{}] [{:6}] {:10} {}",
            marker, fig.id, prio, width_tag, fig.fig_type, status_str, iter_tag,
        );
        let len = line.len().min(tw - 2);

        let style = if is_sel {
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(50, 20, 60))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fig.status.color())
        };
        buf.set_string(x, y, &line[..len], style);
        y += 1;

        // Caption
        let cap = format!("       {}", fig.caption);
        let cap_len = cap.len().min(tw - 2);
        buf.set_string(x, y, &cap[..cap_len], Style::default().fg(Color::DarkGray));
        y += 1;

        // File paths (compact)
        if let Some(pdf) = &fig.pdf_path {
            let fname = pdf.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let file_line = format!("       [*] {}", fname);
            buf.set_string(x, y, &file_line[..file_line.len().min(tw - 2)],
                Style::default().fg(Color::Green));
        } else {
            buf.set_string(x, y, "       [ ] not generated",
                Style::default().fg(Color::DarkGray));
        }
        y += 2;
    }

    // Footer
    let hint = "Up/Down navigate | Enter detail | g generate | v verify | Esc back";
    let hint_y = inner.bottom().saturating_sub(1);
    let hint_x = x + (inner.width.saturating_sub(hint.len() as u16 + 4)) / 2;
    buf.set_string(hint_x, hint_y, hint,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
}

fn render_detail(area: Rect, start_y: u16, buf: &mut Buffer, fig: &FigureEntry) {
    let tw = area.width as usize;
    let x = area.left() + 2;
    let mut y = start_y;

    // Title
    let title = format!("{} -- {}", fig.id.to_uppercase(), fig.fig_type);
    buf.set_string(x, y, &title,
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));
    y += 1;

    // Status with color
    let status_line = format!("Status: {} (iteration {}/{})", fig.status.label(), fig.iteration, fig.max_iterations);
    buf.set_string(x, y, &status_line, Style::default().fg(fig.status.color()));
    y += 1;

    // Priority
    let prio_str = match fig.priority {
        FigurePriority::Essential => "P1 - Essential (system diagram, key result, comparison)",
        FigurePriority::Expected  => "P2 - Expected (setup, calibration, additional results)",
        FigurePriority::Optional  => "P3 - Optional (flowchart, supplementary)",
    };
    buf.set_string(x, y, &format!("Priority: {}", prio_str),
        Style::default().fg(Color::Gray));
    y += 1;

    let width_str = if fig.width == "double" { "Double-column (7.16 in)" } else { "Single-column (3.5 in)" };
    buf.set_string(x, y, &format!("Width: {}", width_str), Style::default().fg(Color::Gray));
    y += 2;

    // Caption
    buf.set_string(x, y, "CAPTION", Style::default().fg(Color::Rgb(100, 200, 255)).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    y += 1;
    // Word-wrap caption
    let cap_width = tw.saturating_sub(6);
    let mut pos = 0;
    while pos < fig.caption.len() && y < area.bottom().saturating_sub(6) {
        let end = (pos + cap_width).min(fig.caption.len());
        buf.set_string(x + 2, y, &fig.caption[pos..end], Style::default().fg(Color::White));
        pos = end;
        y += 1;
    }
    y += 1;

    // Files
    let sep = "─".repeat(tw.saturating_sub(4).min(60));
    buf.set_string(x, y, &sep, Style::default().fg(Color::DarkGray));
    y += 1;

    buf.set_string(x, y, "FILES", Style::default().fg(Color::Rgb(120, 200, 120)).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    y += 1;

    let files = [
        ("PDF", &fig.pdf_path),
        ("PNG", &fig.png_path),
        ("Script", &fig.gp_script),
    ];
    for (label, path) in &files {
        if y >= area.bottom().saturating_sub(3) { break; }
        let (icon, text) = match path {
            Some(p) => ("[*]", p.to_string_lossy().to_string()),
            None => ("[ ]", "not generated".to_string()),
        };
        let line = format!("  {} {}: {}", icon, label, text);
        let color = if path.is_some() { Color::Green } else { Color::DarkGray };
        buf.set_string(x, y, &line[..line.len().min(tw - 2)], Style::default().fg(color));
        y += 1;
    }
    y += 1;

    // Issues
    if !fig.issues.is_empty() && y < area.bottom().saturating_sub(2) {
        buf.set_string(x, y, "ISSUES", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
        y += 1;
        for issue in &fig.issues {
            if y >= area.bottom().saturating_sub(1) { break; }
            let line = format!("  ! {}", issue);
            buf.set_string(x, y, &line[..line.len().min(tw - 2)], Style::default().fg(Color::Yellow));
            y += 1;
        }
    }

    // Footer
    let hint = "Enter close detail | g regenerate | v verify | Esc back to list";
    let hint_y = area.bottom().saturating_sub(1);
    let hint_x = area.left() + (area.width.saturating_sub(hint.len() as u16)) / 2;
    buf.set_string(hint_x, hint_y, hint,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
}
