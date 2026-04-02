use std::path::PathBuf;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Screen};

/// IEEE TIM figure proportion rules:
/// - 6 single-column figures = 1 printed page
/// - 1 display item per ~1,000 words
/// - Target: 5-8 figures for 7-8 page paper
const FIGURES_PER_PAGE: f64 = 0.75;

// ── 5-Stage Iterative Figure Design Pipeline ──
//
// Each figure passes through all 5 stages. Each stage can iterate
// up to 3 times before advancing or failing.
//
//  Stage 1: DRAFT     Generate initial figure from data + GNUPlot script
//  Stage 2: REVIEW    Render to PNG, analyze for readability/label/sizing issues
//  Stage 3: REFINE    Fix identified issues, regenerate with adjusted params
//  Stage 4: VERIFY    Check IEEE formatting (DPI, column width, font size, contrast)
//  Stage 5: PLACE     Integrate into LaTeX, verify placement doesn't break layout

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigureStage {
    Draft,
    Review,
    Refine,
    Verify,
    Place,
    Complete,
}

impl FigureStage {
    fn index(&self) -> usize {
        match self {
            Self::Draft => 0,
            Self::Review => 1,
            Self::Refine => 2,
            Self::Verify => 3,
            Self::Place => 4,
            Self::Complete => 5,
        }
    }

    fn label(&self) -> &str {
        match self {
            Self::Draft    => "DRAFT",
            Self::Review   => "REVIEW",
            Self::Refine   => "REFINE",
            Self::Verify   => "VERIFY",
            Self::Place    => "PLACE",
            Self::Complete => "DONE",
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Draft    => "Generate initial figure from data + GNUPlot script",
            Self::Review   => "Render PNG, analyze readability, labels, sizing, alignment",
            Self::Refine   => "Fix issues, adjust params, regenerate",
            Self::Verify   => "Check IEEE rules: DPI >=300, column width, font 8-12pt, contrast",
            Self::Place    => "Integrate into LaTeX, verify placement and text flow",
            Self::Complete => "Figure accepted and placed in paper",
        }
    }

    fn all() -> &'static [FigureStage] {
        &[Self::Draft, Self::Review, Self::Refine, Self::Verify, Self::Place]
    }
}

/// Status of a single stage within a figure's pipeline
#[derive(Debug, Clone)]
pub struct StageStatus {
    pub stage: FigureStage,
    pub status: StageResult,
    pub iteration: u8,
    pub max_iterations: u8,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageResult {
    Pending,
    InProgress,
    Passed,
    NeedsIteration,
    Failed,
}

impl StageResult {
    fn icon(&self) -> &str {
        match self {
            Self::Pending => " ",
            Self::InProgress => ">",
            Self::Passed => "*",
            Self::NeedsIteration => "~",
            Self::Failed => "!",
        }
    }
    fn color(&self) -> Color {
        match self {
            Self::Pending => Color::DarkGray,
            Self::InProgress => Color::Yellow,
            Self::Passed => Color::Green,
            Self::NeedsIteration => Color::Rgb(255, 165, 0),
            Self::Failed => Color::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FigurePriority {
    Essential,
    Expected,
    Optional,
}

#[derive(Debug, Clone)]
pub struct FigureEntry {
    pub id: String,
    pub caption: String,
    pub fig_type: String,
    pub priority: FigurePriority,
    pub width: &'static str, // "single" or "double"
    pub pdf_path: Option<PathBuf>,
    pub png_path: Option<PathBuf>,
    pub gp_script: Option<PathBuf>,
    pub current_stage: FigureStage,
    pub stages: Vec<StageStatus>,
}

impl FigureEntry {
    fn new(id: &str, caption: &str, fig_type: &str, priority: FigurePriority, width: &'static str) -> Self {
        let stages = FigureStage::all()
            .iter()
            .map(|s| StageStatus {
                stage: *s,
                status: StageResult::Pending,
                iteration: 0,
                max_iterations: 3,
                issues: vec![],
            })
            .collect();

        Self {
            id: id.to_string(),
            caption: caption.to_string(),
            fig_type: fig_type.to_string(),
            priority,
            width,
            pdf_path: None,
            png_path: None,
            gp_script: None,
            current_stage: FigureStage::Draft,
            stages,
        }
    }

    fn stages_done(&self) -> usize {
        self.stages.iter().filter(|s| s.status == StageResult::Passed).count()
    }

    fn is_complete(&self) -> bool {
        self.current_stage == FigureStage::Complete
    }

    fn total_iterations(&self) -> u8 {
        self.stages.iter().map(|s| s.iteration).sum()
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
        let figures = Self::build_figures(app);
        Self {
            figures,
            selected: 0,
            page_count: 8,
            show_detail: false,
        }
    }

    fn build_figures(app: &App) -> Vec<FigureEntry> {
        let fig_dir = app.config.output_dir.join("figures");
        let mut figures = vec![
            FigureEntry::new("fig1", "Hardware cost comparison of existing leak detection systems and the proposed GASLEAD system.", "bar", FigurePriority::Essential, "single"),
            FigureEntry::new("fig2", "UW IAC compressed air energy savings from leak detection recommendations (2022--2024).", "bar+line", FigurePriority::Essential, "single"),
            FigureEntry::new("fig3", "Simulated CEM transfer function H(f) for steel and PVC pipe segments, showing material-dependent resonance characteristics.", "line", FigurePriority::Essential, "single"),
            FigureEntry::new("fig4", "Simulated detection performance (ROC curves) comparing GASLEAD with CEM calibration, without CEM, and fixed-threshold approaches.", "line", FigurePriority::Essential, "single"),
            FigureEntry::new("fig5", "GASLEAD system architecture showing distributed sensor nodes, CEM, LoRa gateway, and backend infrastructure.", "block", FigurePriority::Essential, "double"),
        ];

        // Scan disk for existing files and update stage status
        for fig in &mut figures {
            if let Ok(entries) = std::fs::read_dir(&fig_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&fig.id) {
                        if name.ends_with(".pdf") {
                            fig.pdf_path = Some(entry.path());
                        } else if name.ends_with(".png") {
                            fig.png_path = Some(entry.path());
                        } else if name.ends_with(".gp") {
                            fig.gp_script = Some(entry.path());
                        }
                    }
                }
            }

            // If files exist, advance stages based on what's on disk
            if fig.pdf_path.is_some() && fig.png_path.is_some() {
                // Draft stage passed (files generated)
                fig.stages[0].status = StageResult::Passed;
                fig.stages[0].iteration = 1;
                // Review: needs iteration (figures are messy, unreviewed)
                fig.stages[1].status = StageResult::NeedsIteration;
                fig.stages[1].iteration = 1;
                fig.stages[1].issues = vec!["Awaiting visual review".to_string()];
                fig.current_stage = FigureStage::Review;
            }
        }

        figures
    }

    fn recommended_count(&self) -> usize {
        (self.page_count as f64 * FIGURES_PER_PAGE).round() as usize
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.figures.len() { self.selected += 1; }
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

// ── Rendering ──

pub fn render(area: Rect, buf: &mut Buffer, state: &FiguresState, _app: &App) {
    let block = Block::default()
        .title(" Research Pipeline -- Figures (5-Stage Design) ")
        .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 40, 100)));
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 8 || inner.width < 50 { return; }

    let tw = inner.width as usize;
    let mut y = inner.top() + 1;
    let x = inner.left() + 2;

    // Header stats
    let total = state.figures.len();
    let complete = state.figures.iter().filter(|f| f.is_complete()).count();
    let in_review = state.figures.iter().filter(|f| f.current_stage == FigureStage::Review).count();
    let recommended = state.recommended_count();
    let header = format!(
        "{}/{} complete | {} in review | {} recommended for {} pages",
        complete, total, in_review, recommended, state.page_count,
    );
    buf.set_string(x, y, &header[..header.len().min(tw - 4)],
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    y += 1;

    // Stage legend
    let legend = "Stages: DRAFT > REVIEW > REFINE > VERIFY > PLACE";
    buf.set_string(x, y, legend, Style::default().fg(Color::DarkGray));
    y += 2;

    if state.show_detail {
        if let Some(fig) = state.figures.get(state.selected) {
            render_detail(inner, y, buf, fig, tw);
        }
        return;
    }

    // Figure list with mini stage pipeline
    for (i, fig) in state.figures.iter().enumerate() {
        if y + 4 >= inner.bottom() { break; }

        let is_sel = state.selected == i;
        let marker = if is_sel { ">>" } else { "  " };

        // Line 1: ID, type, priority, current stage
        let prio = match fig.priority {
            FigurePriority::Essential => "P1",
            FigurePriority::Expected  => "P2",
            FigurePriority::Optional  => "P3",
        };
        let width_tag = if fig.width == "double" { "2col" } else { "1col" };
        let line1 = format!(
            "{} {} [{}] [{}] [{:6}]  Stage: {}  (iters: {})",
            marker, fig.id, prio, width_tag, fig.fig_type,
            fig.current_stage.label(), fig.total_iterations(),
        );
        let style = if is_sel {
            Style::default().fg(Color::White).bg(Color::Rgb(50, 20, 60)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        buf.set_string(x, y, &line1[..line1.len().min(tw - 2)], style);
        y += 1;

        // Line 2: Mini pipeline  *DRA >> >REV >> ~REF >> VER >>  PLA
        let mut px = x + 4;
        for ss in &fig.stages {
            if px + 10 >= inner.right() { break; }
            let tag = format!("{}{}", ss.status.icon(), &ss.stage.label()[..3]);
            let stage_style = if ss.stage == fig.current_stage {
                Style::default().fg(ss.status.color()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(ss.status.color())
            };
            buf.set_string(px, y, &tag, stage_style);
            px += tag.len() as u16 + 1;
            if ss.stage != FigureStage::Place {
                buf.set_string(px, y, ">>", Style::default().fg(Color::DarkGray));
                px += 3;
            }
        }
        y += 1;

        // Line 3: Caption (truncated)
        let cap = format!("       {}", fig.caption);
        let cap_len = cap.len().min(tw - 2);
        buf.set_string(x, y, &cap[..cap_len], Style::default().fg(Color::DarkGray));
        y += 2;
    }

    // Footer
    let hint = "Up/Down navigate | Enter detail | Esc back";
    let hint_y = inner.bottom().saturating_sub(1);
    let hint_x = x + (inner.width.saturating_sub(hint.len() as u16 + 4)) / 2;
    buf.set_string(hint_x, hint_y, hint,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
}

fn render_detail(area: Rect, start_y: u16, buf: &mut Buffer, fig: &FigureEntry, tw: usize) {
    let x = area.left() + 2;
    let mut y = start_y;

    // Title
    let title = format!("{} -- {} ({})", fig.id.to_uppercase(), fig.fig_type,
        if fig.width == "double" { "double-column, 7.16 in" } else { "single-column, 3.5 in" });
    buf.set_string(x, y, &title,
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));
    y += 2;

    // 5-stage pipeline detail
    buf.set_string(x, y, "DESIGN PIPELINE",
        Style::default().fg(Color::Rgb(100, 200, 255)).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    y += 1;

    for ss in &fig.stages {
        if y + 3 >= area.bottom().saturating_sub(6) { break; }

        let is_current = ss.stage == fig.current_stage;
        let arrow = if is_current { ">>" } else { "  " };
        let icon = ss.status.icon();
        let iter_str = if ss.iteration > 0 {
            format!("(iter {}/{})", ss.iteration, ss.max_iterations)
        } else {
            String::new()
        };

        // Stage header
        let stage_line = format!(
            "{} [{}] {:6} {:16} {}",
            arrow, icon, ss.stage.label(), ss.stage.description(), iter_str,
        );
        let stage_style = if is_current {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else if ss.status == StageResult::Passed {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(ss.status.color())
        };
        buf.set_string(x, y, &stage_line[..stage_line.len().min(tw - 2)], stage_style);
        y += 1;

        // Issues for this stage
        for issue in &ss.issues {
            if y >= area.bottom().saturating_sub(6) { break; }
            let issue_line = format!("          ! {}", issue);
            buf.set_string(x, y, &issue_line[..issue_line.len().min(tw - 2)],
                Style::default().fg(Color::Yellow));
            y += 1;
        }
    }
    y += 1;

    // Files section
    let sep = "─".repeat(tw.saturating_sub(4).min(60));
    buf.set_string(x, y, &sep, Style::default().fg(Color::DarkGray));
    y += 1;

    buf.set_string(x, y, "FILES",
        Style::default().fg(Color::Rgb(120, 200, 120)).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    y += 1;

    let files = [
        ("PDF   ", &fig.pdf_path),
        ("PNG   ", &fig.png_path),
        ("Script", &fig.gp_script),
    ];
    for (label, path) in &files {
        if y >= area.bottom().saturating_sub(3) { break; }
        let (icon, text) = match path {
            Some(p) => ("[*]", p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()),
            None => ("[ ]", "not generated".to_string()),
        };
        let line = format!("  {} {}: {}", icon, label, text);
        let color = if path.is_some() { Color::Green } else { Color::DarkGray };
        buf.set_string(x, y, &line[..line.len().min(tw - 2)], Style::default().fg(color));
        y += 1;
    }
    y += 1;

    // Caption
    buf.set_string(x, y, "CAPTION",
        Style::default().fg(Color::Rgb(200, 200, 100)).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    y += 1;
    let cap_w = tw.saturating_sub(6);
    let mut pos = 0;
    while pos < fig.caption.len() && y < area.bottom().saturating_sub(2) {
        let end = (pos + cap_w).min(fig.caption.len());
        buf.set_string(x + 2, y, &fig.caption[pos..end], Style::default().fg(Color::White));
        pos = end;
        y += 1;
    }

    // Footer
    let hint = "Enter close | Esc back to list";
    let hint_y = area.bottom().saturating_sub(1);
    let hint_x = area.left() + (area.width.saturating_sub(hint.len() as u16)) / 2;
    buf.set_string(hint_x, hint_y, hint,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
}
