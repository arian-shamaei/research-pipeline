use std::fs;

use ratatui::prelude::*;
use ratatui::widgets::*;

const RELOAD_INTERVAL: u64 = 10;

// -- Data structures (mirrors neil-blueprint-v6/src/planes/paper_pipeline.rs) --

#[derive(Clone, Debug, Default)]
struct ScriptInfo {
    file: String,
    function: String,
    params: Vec<String>,
    last_run: String,
    duration_ms: u64,
    success: bool,
    output_size: u64,
}

#[derive(Clone, Debug, Default)]
struct StageMetrics {
    entries: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default)]
struct StageInfo {
    id: String,
    label: String,
    description: String,
    processes: Vec<String>,
    scripts: Vec<ScriptInfo>,
    outputs: Vec<String>,
    metrics: StageMetrics,
}

#[derive(Clone, Debug, Default)]
struct PaperStageStatus {
    status: String,
    artifacts: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct ActivePaper {
    name: String,
    source: String,
    code: String,
    stage: String,
    stage_progress: f64,
    priority: f64,
    problems: Vec<String>,
    last_activity: String,
    stages: Vec<(String, PaperStageStatus)>,
}

#[derive(Clone, Debug, Default)]
struct ComponentInfo {
    name: String,
    status: String,
}

#[derive(Clone, Debug, Default)]
struct PipelineData {
    engine_version: String,
    stages: Vec<StageInfo>,
    papers: Vec<ActivePaper>,
    components: Vec<ComponentInfo>,
    problem_coverage: Vec<(String, u64)>,
}

pub struct PaperPipelinePlane {
    data: PipelineData,
    tick_count: u64,
    loaded: bool,
    selected_stage: usize,
    selected_item: usize,
    show_item_detail: bool,
}

impl PaperPipelinePlane {
    pub fn new() -> Self {
        let mut plane = Self {
            data: PipelineData::default(),
            tick_count: 0,
            loaded: false,
            selected_stage: 0,
            selected_item: 0,
            show_item_detail: false,
        };
        plane.load_data();
        plane
    }

    fn load_data(&mut self) {
        let workspace = std::env::var("OPENCLAW_WORKSPACE")
            .unwrap_or_else(|_| r"C:\Users\Administrator\.openclaw\workspace".to_string());
        let path = std::path::Path::new(&workspace).join(".paper_pipeline.json");
        if !path.exists() {
            self.loaded = false;
            return;
        }
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                self.loaded = true;
                self.data = Self::parse_data(&val);
            }
        }
    }

    fn parse_data(val: &serde_json::Value) -> PipelineData {
        let mut data = PipelineData::default();

        data.engine_version = val
            .get("engine_version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if let Some(stages) = val.get("pipeline_stages").and_then(|v| v.as_array()) {
            for s in stages {
                let mut scripts = Vec::new();
                if let Some(sc_arr) = s.get("scripts").and_then(|v| v.as_array()) {
                    for sc in sc_arr {
                        scripts.push(ScriptInfo {
                            file: sc
                                .get("file")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            function: sc
                                .get("function")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            params: sc
                                .get("params")
                                .and_then(|v| v.as_array())
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|t| t.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default(),
                            last_run: sc
                                .get("last_run")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            duration_ms: sc
                                .get("duration_ms")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0),
                            success: sc
                                .get("success")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            output_size: sc
                                .get("output_size")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0),
                        });
                    }
                }
                let mut metrics = StageMetrics::default();
                if let Some(m) = s.get("metrics").and_then(|v| v.as_object()) {
                    for (k, v) in m {
                        let val_str = if let Some(s) = v.as_str() {
                            s.to_string()
                        } else if let Some(n) = v.as_f64() {
                            format!("{}", n)
                        } else if let Some(b) = v.as_bool() {
                            if b {
                                "Yes".into()
                            } else {
                                "No".into()
                            }
                        } else {
                            v.to_string()
                        };
                        metrics.entries.push((k.clone(), val_str));
                    }
                }

                data.stages.push(StageInfo {
                    id: s
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    label: s
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: s
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    processes: s
                        .get("processes")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|t| t.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    scripts,
                    outputs: s
                        .get("outputs")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|t| t.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    metrics,
                });
            }
        }

        if let Some(comps) = val.get("components").and_then(|v| v.as_object()) {
            for (name, info) in comps {
                data.components.push(ComponentInfo {
                    name: name.clone(),
                    status: info
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }

        if let Some(coverage) = val.get("problem_coverage").and_then(|v| v.as_object()) {
            for (k, v) in coverage {
                data.problem_coverage.push((k.clone(), v.as_u64().unwrap_or(0)));
            }
            data.problem_coverage.sort_by(|a, b| b.1.cmp(&a.1));
        }

        if let Some(papers) = val.get("active_papers").and_then(|v| v.as_array()) {
            let stage_order = [
                "SCOUT",
                "EVALUATE",
                "READ",
                "PROTOTYPE",
                "INTEGRATE",
                "DOCUMENT",
                "VERIFY",
                "YBR_DOCS",
            ];
            for p in papers {
                let mut paper = ActivePaper {
                    name: p
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    source: p
                        .get("source")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    code: p
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    stage: p
                        .get("stage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    stage_progress: p
                        .get("stage_progress")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    priority: p
                        .get("priority")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    problems: p
                        .get("problems")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|t| t.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    last_activity: p
                        .get("last_activity")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    stages: Vec::new(),
                };

                if let Some(stages) = p.get("stages").and_then(|v| v.as_object()) {
                    for stage_id in &stage_order {
                        if let Some(st) = stages.get(*stage_id) {
                            paper.stages.push((
                                stage_id.to_string(),
                                PaperStageStatus {
                                    status: st
                                        .get("status")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    artifacts: st
                                        .get("artifacts")
                                        .and_then(|v| v.as_array())
                                        .map(|a| {
                                            a.iter()
                                                .filter_map(|t| t.as_str().map(String::from))
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                },
                            ));
                        }
                    }
                }

                data.papers.push(paper);
            }
        }

        data
    }

    fn stage_color(status: &str) -> Color {
        match status {
            "done" => Color::Green,
            "active" => Color::Yellow,
            "pending" => Color::DarkGray,
            _ => Color::Gray,
        }
    }

    fn stage_icon(status: &str) -> &'static str {
        match status {
            "done" => "*",
            "active" => ">",
            "pending" => "o",
            _ => "?",
        }
    }

    /// Render the pipeline flow boxes: |*SCOUT|==>|*EVAL|==>|*READ|==>| PROTO|==>| INTEG|
    fn render_pipeline_flow(
        &self,
        paper: &ActivePaper,
        area_left: u16,
        area_right: u16,
        y_start: u16,
        buf: &mut Buffer,
        highlight_idx: Option<usize>,
    ) -> u16 {
        let tw = (area_right - area_left) as usize;
        let mut y = y_start;

        // Paper title line
        let short_name = if paper.name.len() > tw.saturating_sub(20) {
            format!(
                "{}..",
                &paper.name[..tw.saturating_sub(22).min(paper.name.len())]
            )
        } else {
            paper.name.clone()
        };
        let header = format!(
            " [{}] {} (P:{:.0})",
            paper.code, short_name, paper.priority
        );
        let hdr_len = header.len().min(tw);
        buf.set_string(
            area_left,
            y,
            &header[..hdr_len],
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        let labels: Vec<(&str, &str)> = paper
            .stages
            .iter()
            .map(|(id, st)| {
                let short = match id.as_str() {
                    "SCOUT" => "SCOUT",
                    "EVALUATE" => "EVAL",
                    "READ" => "READ",
                    "PROTOTYPE" => "PROTO",
                    "INTEGRATE" => "INTEG",
                    "DOCUMENT" => "DOC",
                    "VERIFY" => "VERFY",
                    "YBR_DOCS" => "YBR",
                    _ => id.as_str(),
                };
                (short, st.status.as_str())
            })
            .collect();

        let n = labels.len();
        if n == 0 {
            return y - y_start;
        }

        let arrow_str = "==>";
        let arrow_len = arrow_str.len();
        let total_arrow_space = (n - 1) * arrow_len;
        let available_for_boxes = tw.saturating_sub(total_arrow_space + 2);
        let box_inner = (available_for_boxes / n).max(5).min(9);
        let box_outer = box_inner + 2;

        // Top border
        let mut x = area_left + 1;
        for (i, _) in labels.iter().enumerate() {
            if (x as usize + box_outer) > tw + area_left as usize {
                break;
            }
            let top = format!("+{}+", "-".repeat(box_inner));
            let border_style = if highlight_idx == Some(i) {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            buf.set_string(x, y, &top, border_style);
            x += box_outer as u16;
            if i < n - 1 {
                x += arrow_len as u16;
            }
        }
        y += 1;

        // Middle line with labels
        x = area_left + 1;
        for (i, (label, status)) in labels.iter().enumerate() {
            if (x as usize + box_outer) > tw + area_left as usize {
                break;
            }
            let color = if highlight_idx == Some(i) {
                Color::White
            } else {
                Self::stage_color(status)
            };
            let marker = Self::stage_icon(status);
            let content = format!("{}{}", marker, label);
            let padding_total = box_inner.saturating_sub(content.len());
            let pad_left = padding_total / 2;
            let pad_right = padding_total - pad_left;
            let cell = format!(
                "|{}{}{}|",
                " ".repeat(pad_left),
                content,
                " ".repeat(pad_right)
            );

            let style = if highlight_idx == Some(i) {
                Style::default()
                    .fg(color)
                    .bg(Color::Rgb(50, 20, 60))
                    .add_modifier(Modifier::BOLD)
            } else if *status == "active" {
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };
            buf.set_string(x, y, &cell, style);
            x += box_outer as u16;

            if i < n - 1 {
                let arrow_color = if *status == "done" {
                    Color::Green
                } else {
                    Color::DarkGray
                };
                buf.set_string(x, y, arrow_str, Style::default().fg(arrow_color));
                x += arrow_len as u16;
            }
        }
        y += 1;

        // Bottom border
        x = area_left + 1;
        for (i, _) in labels.iter().enumerate() {
            if (x as usize + box_outer) > tw + area_left as usize {
                break;
            }
            let bot = format!("+{}+", "-".repeat(box_inner));
            let border_style = if highlight_idx == Some(i) {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            buf.set_string(x, y, &bot, border_style);
            x += box_outer as u16;
            if i < n - 1 {
                x += arrow_len as u16;
            }
        }
        y += 1;

        y - y_start
    }

    fn render_progress_bar(
        x: u16,
        y: u16,
        width: u16,
        progress: f64,
        color: Color,
        buf: &mut Buffer,
    ) {
        let filled = ((width as f64) * progress.clamp(0.0, 1.0)) as u16;
        let empty = width - filled;
        let bar = format!(
            "[{}{}] {:>3.0}%",
            "#".repeat(filled as usize),
            ".".repeat(empty as usize),
            progress * 100.0
        );
        buf.set_string(x, y, &bar, Style::default().fg(color));
    }

    /// Render detailed view for a selected stage
    fn render_stage_detail(
        &self,
        stage: &StageInfo,
        prop_status: Option<&str>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let tw = area.width as usize;
        let mut y = area.y;
        let sel = self.selected_item;

        let status_str = prop_status.unwrap_or("pending");
        let status_color = Self::stage_color(status_str);
        let status_icon = match status_str {
            "done" => "[*]",
            "active" => "[>]",
            _ => "[ ]",
        };

        // Stage header
        let hdr = format!("{} {} -- {}", status_icon, stage.id, stage.description);
        let hdr_len = hdr.len().min(tw);
        buf.set_string(
            area.x,
            y,
            &hdr[..hdr_len],
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        // Progress bar
        let progress = match status_str {
            "done" => 1.0,
            "active" => {
                let total = stage.processes.len() + stage.scripts.len();
                if total == 0 {
                    0.5
                } else {
                    let done = stage.scripts.iter().filter(|s| s.success).count();
                    done as f64 / total as f64
                }
            }
            _ => 0.0,
        };
        buf.set_string(
            area.x + 1,
            y,
            "Progress: ",
            Style::default().fg(Color::Gray),
        );
        Self::render_progress_bar(
            area.x + 11,
            y,
            30.min(area.width.saturating_sub(13)),
            progress,
            status_color,
            buf,
        );
        y += 1;

        // Metrics bar
        if !stage.metrics.entries.is_empty() && y + 1 < area.bottom() {
            let mut mx = area.x + 1;
            buf.set_string(
                mx,
                y,
                "METRICS ",
                Style::default()
                    .fg(Color::Rgb(180, 140, 255))
                    .add_modifier(Modifier::BOLD),
            );
            mx += 8;
            for (label, value) in &stage.metrics.entries {
                let metric_str = format!("{}:{} ", label, value);
                if mx + metric_str.len() as u16 >= area.right() {
                    y += 1;
                    mx = area.x + 9;
                    if y >= area.bottom() {
                        break;
                    }
                }
                buf.set_string(
                    mx,
                    y,
                    label,
                    Style::default().fg(Color::Rgb(140, 100, 200)),
                );
                mx += label.len() as u16;
                buf.set_string(mx, y, ":", Style::default().fg(Color::DarkGray));
                mx += 1;
                buf.set_string(
                    mx,
                    y,
                    value,
                    Style::default()
                        .fg(Color::Rgb(200, 180, 255))
                        .add_modifier(Modifier::BOLD),
                );
                mx += value.len() as u16 + 2;
            }
            y += 1;
        }

        // Separator
        let sep: String = "-".repeat(tw.min(60));
        buf.set_string(area.x, y, &sep, Style::default().fg(Color::DarkGray));
        y += 1;

        // -- GROUND TRUTH MODE --
        if self.show_item_detail {
            buf.set_string(
                area.x + 1,
                y,
                "GROUND TRUTH",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            y += 1;

            let proc_count = stage.processes.len();
            if sel < proc_count {
                if let Some(proc_name) = stage.processes.get(sel) {
                    let lines = [
                        "  type:        Process".to_string(),
                        format!("  name:        {:?}", proc_name),
                        format!("  stage:       {:?} ({})", stage.id, stage.description),
                        format!("  index:       {}/{}", sel + 1, proc_count),
                        format!("  status:      {:?}", status_str),
                    ];
                    for line in &lines {
                        if y >= area.bottom().saturating_sub(1) {
                            break;
                        }
                        let disp = if line.len() > tw {
                            &line[..tw]
                        } else {
                            line.as_str()
                        };
                        buf.set_string(area.x, y, disp, Style::default().fg(Color::DarkGray));
                        y += 1;
                    }
                }
                if !stage.outputs.is_empty() && y < area.bottom().saturating_sub(1) {
                    let out_line = format!("  outputs:     {:?}", stage.outputs);
                    let disp = if out_line.len() > tw {
                        &out_line[..tw]
                    } else {
                        out_line.as_str()
                    };
                    buf.set_string(area.x, y, disp, Style::default().fg(Color::DarkGray));
                    y += 1;
                }
            } else {
                let script_idx = sel - proc_count;
                if let Some(sc) = stage.scripts.get(script_idx) {
                    let lines = [
                        "  type:        Script".to_string(),
                        format!("  file:        {:?}", sc.file),
                        format!("  function:    {:?}", sc.function),
                        format!("  params:      {:?}", sc.params),
                        format!("  last_run:    {:?}", sc.last_run),
                        format!("  duration_ms: {}", sc.duration_ms),
                        format!("  success:     {}", sc.success),
                        format!("  output_size: {}", sc.output_size),
                    ];
                    for line in &lines {
                        if y >= area.bottom().saturating_sub(1) {
                            break;
                        }
                        let disp = if line.len() > tw {
                            &line[..tw]
                        } else {
                            line.as_str()
                        };
                        buf.set_string(area.x, y, disp, Style::default().fg(Color::DarkGray));
                        y += 1;
                    }
                }
            }

            let hint_str = "<-/-> stage  |  up/down item  |  Enter close detail  |  ESC close";
            let hint_y = area.bottom().saturating_sub(1);
            let hint_x = area.x + (area.width.saturating_sub(hint_str.len() as u16)) / 2;
            buf.set_string(
                hint_x,
                hint_y,
                hint_str,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            );
            return;
        }

        // -- PROCESSES section --
        buf.set_string(
            area.x + 1,
            y,
            "PROCESSES",
            Style::default()
                .fg(Color::Rgb(100, 200, 255))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
        y += 1;

        for (i, proc) in stage.processes.iter().enumerate() {
            if y >= area.bottom().saturating_sub(2) {
                break;
            }
            let is_selected = sel == i;
            let check = if status_str == "done" {
                "[*]"
            } else if status_str == "active" && i < stage.processes.len() / 2 {
                "[*]"
            } else if status_str == "active" {
                "[>]"
            } else {
                "[ ]"
            };

            let marker = if is_selected { ">>" } else { "  " };
            let bullet = format!("{} {} {}", marker, check, proc);
            let len = bullet.len().min(tw);

            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(50, 20, 60))
                    .add_modifier(Modifier::BOLD)
            } else if check == "[*]" {
                Style::default().fg(Color::Green)
            } else if check == "[>]" {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            buf.set_string(area.x, y, &bullet[..len], style);
            y += 1;
        }
        y += 1;

        // -- SCRIPTS section --
        if y + 2 < area.bottom().saturating_sub(2) && !stage.scripts.is_empty() {
            buf.set_string(
                area.x + 1,
                y,
                "SCRIPTS",
                Style::default()
                    .fg(Color::Rgb(120, 200, 120))
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            );
            y += 1;

            let proc_count = stage.processes.len();
            for (i, sc) in stage.scripts.iter().enumerate() {
                if y + 2 >= area.bottom().saturating_sub(2) {
                    break;
                }
                let item_idx = proc_count + i;
                let is_selected = sel == item_idx;
                let marker = if is_selected { ">>" } else { "  " };

                let run_info = if !sc.last_run.is_empty() {
                    let status_ch = if sc.success { "*" } else { "x" };
                    let dur = if sc.duration_ms > 0 {
                        format!(" {}ms", sc.duration_ms)
                    } else {
                        String::new()
                    };
                    let size = if sc.output_size > 0 {
                        format!(" out:{}", sc.output_size)
                    } else {
                        String::new()
                    };
                    format!(" [{}{}{}]", status_ch, dur, size)
                } else {
                    " [not run]".to_string()
                };

                let line1 = format!("{} {} -> {}{}", marker, sc.file, sc.function, run_info);
                let len1 = line1.len().min(tw);

                let style = if is_selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(30, 50, 30))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(120, 200, 120))
                };
                buf.set_string(area.x, y, &line1[..len1], style);
                y += 1;

                if !sc.params.is_empty() && y < area.bottom().saturating_sub(2) {
                    let params = format!("     params: [{}]", sc.params.join(", "));
                    let len_p = params.len().min(tw);
                    let param_style = if is_selected {
                        Style::default().fg(Color::Rgb(150, 220, 150))
                    } else {
                        Style::default().fg(Color::Rgb(60, 120, 60))
                    };
                    buf.set_string(area.x, y, &params[..len_p], param_style);
                    y += 1;
                }
            }
            y += 1;
        }

        // -- OUTPUTS section --
        if y + 2 < area.bottom().saturating_sub(1) && !stage.outputs.is_empty() {
            buf.set_string(
                area.x + 1,
                y,
                "OUTPUTS",
                Style::default()
                    .fg(Color::Rgb(200, 200, 100))
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            );
            y += 1;
            for out in &stage.outputs {
                if y >= area.bottom().saturating_sub(1) {
                    break;
                }
                let line = format!("  => {}", out);
                let len = line.len().min(tw);
                buf.set_string(
                    area.x,
                    y,
                    &line[..len],
                    Style::default().fg(Color::Rgb(200, 200, 100)),
                );
                y += 1;
            }
        }

        // Footer hint
        let hint_str = "<-/-> stage  |  up/down item  |  Enter ground truth  |  ESC close";
        let hint_y = area.bottom().saturating_sub(1);
        let hint_x = area.x + (area.width.saturating_sub(hint_str.len() as u16)) / 2;
        buf.set_string(
            hint_x,
            hint_y,
            hint_str,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        );
    }

    // -- Public interface (replaces Plane trait methods) --

    pub fn update(&mut self) {
        self.tick_count += 1;
        if self.tick_count % RELOAD_INTERVAL == 0 {
            self.load_data();
        }
    }

    pub fn handle_arrow(&mut self, right: bool) -> bool {
        let n = self.data.stages.len();
        if n == 0 {
            return false;
        }
        if right {
            self.selected_stage = (self.selected_stage + 1) % n;
        } else {
            self.selected_stage = if self.selected_stage == 0 {
                n - 1
            } else {
                self.selected_stage - 1
            };
        }
        self.selected_item = 0;
        self.show_item_detail = false;
        true
    }

    pub fn handle_enter(&mut self) -> bool {
        self.show_item_detail = !self.show_item_detail;
        true
    }

    pub fn scroll_by(&mut self, delta: i32) {
        self.show_item_detail = false;
        if let Some(stage) = self.data.stages.get(self.selected_stage) {
            let max_items = stage.processes.len() + stage.scripts.len();
            if max_items == 0 {
                return;
            }
            let new_val = self.selected_item as i32 + delta;
            self.selected_item = new_val.clamp(0, max_items as i32) as usize;
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 || area.width < 20 {
            return;
        }

        let d = &self.data;
        let active_count = d.components.iter().filter(|c| c.status == "active").count();
        let total_count = d.components.len();
        let papers_tracked = d.papers.len();

        let title = format!(
            " Research Pipeline v{}  |  {}/{} tools  |  {} papers tracked ",
            d.engine_version, active_count, total_count, papers_tracked,
        );

        let block = Block::default()
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 40, 100)));
        let inner = block.inner(area);
        block.render(area, buf);

        if !self.loaded {
            if inner.height > 2 {
                let msg = "No data -- waiting for .paper_pipeline.json";
                let x = inner.left() + (inner.width.saturating_sub(msg.len() as u16)) / 2;
                buf.set_string(
                    x,
                    inner.top() + inner.height / 2,
                    msg,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                );
            }
            return;
        }

        let tw = inner.width as usize;
        let mut y = inner.top();

        // Pipeline flows for top papers (show first 2-3 depending on space)
        let max_papers = if inner.height > 20 { 3 } else { 2 };
        for paper in d.papers.iter().take(max_papers) {
            if y + 6 >= inner.bottom() {
                break;
            }
            let lines_used =
                self.render_pipeline_flow(paper, inner.left(), inner.right(), y, buf, None);
            y += lines_used;

            // Detail line
            let done_stages = paper
                .stages
                .iter()
                .filter(|(_, st)| st.status == "done")
                .count();
            let probs: Vec<&str> = paper
                .problems
                .iter()
                .map(|s| match s.as_str() {
                    "identity_drift" => "ident",
                    "self_improvement" => "self",
                    "memory_persistence" => "mem",
                    "retrieval_quality" => "retr",
                    "planning_execution" => "plan",
                    "reasoning_chain_loss" => "reason",
                    "emotional_intelligence" => "emot",
                    _ => s.as_str(),
                })
                .collect();
            let detail = format!(
                "  {}/{} stages | {} | {}",
                done_stages,
                paper.stages.len(),
                probs.join(" "),
                paper.last_activity
            );
            let det_len = detail.len().min(tw);
            buf.set_string(
                inner.left(),
                y,
                &detail[..det_len],
                Style::default().fg(Color::DarkGray),
            );
            y += 2;
        }

        // Separator
        if y < inner.bottom() {
            let sep: String = "-".repeat(tw);
            buf.set_string(inner.left(), y, &sep, Style::default().fg(Color::DarkGray));
            y += 1;
        }

        // Component grid
        if y < inner.bottom() {
            buf.set_string(
                inner.left() + 1,
                y,
                "TOOLS:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
            let mut x = inner.left() + 8;
            for c in &d.components {
                let short = match c.name.as_str() {
                    "scout" => "SCT",
                    "evaluator" => "EVL",
                    "reader" => "RDR",
                    "tracker" => "TRK",
                    "pipeline" => "PIP",
                    "prototype_runner" => "PRT",
                    "integrator" => "INT",
                    // Shared tools (same in proposal pipeline)
                    "citation_engine" => "CIT",
                    "figure_engine" => "FIG",
                    "adversarial_reviewer" => "ADV",
                    "humanizer" => "HUM",
                    "gpt_checker" => "GPT",
                    "qa_auditor" => "QAA",
                    "ybr_documenter" => "YBR",
                    "chart_builder" => "CHT",
                    "research_web" => "WEB",
                    // Research-only tools
                    "doc_compiler" => "DOC",
                    "data_collector" => "DAT",
                    "experiment_runner" => "EXP",
                    "analysis" => "ANL",
                    "reproducibility_checker" => "RPR",
                    "ablation_runner" => "ABL",
                    "quality_scorer" => "QSC",
                    "benchmark_extractor" => "BEX",
                    "benchmark_runner" => "BRN",
                    "benchmark_comparator" => "BCM",
                    _ => &c.name[..3.min(c.name.len())],
                };
                let active = c.status == "active";
                if x + short.len() as u16 + 3 >= inner.right() {
                    y += 1;
                    x = inner.left() + 8;
                    if y >= inner.bottom() {
                        break;
                    }
                }
                let (icon, color) = if active {
                    ("*", Color::Green)
                } else {
                    ("o", Color::DarkGray)
                };
                buf.set_string(
                    x,
                    y,
                    &format!("{}{}", icon, short),
                    Style::default().fg(color),
                );
                x += short.len() as u16 + 2;
            }
        }
    }

    pub fn render_detail(&self, area: Rect, buf: &mut Buffer) {
        if !self.loaded || area.height < 10 || area.width < 40 {
            self.render(area, buf);
            return;
        }

        let d = &self.data;

        let block = Block::default()
            .title(" Research Pipeline -- Stage Detail View ")
            .title_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 40, 100)));
        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.top();

        // Render pipeline flow for the first paper (with selected highlight)
        if let Some(paper) = d.papers.first() {
            let lines_used = self.render_pipeline_flow(
                paper,
                inner.left(),
                inner.right(),
                y,
                buf,
                Some(self.selected_stage),
            );
            y += lines_used + 1;

            // Overall progress
            let done_stages = paper
                .stages
                .iter()
                .filter(|(_, st)| st.status == "done")
                .count();
            let total = paper.stages.len().max(1);
            let overall_progress = done_stages as f64 / total as f64;
            buf.set_string(
                inner.left() + 1,
                y,
                "Overall: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
            Self::render_progress_bar(
                inner.left() + 10,
                y,
                40.min(inner.width - 12),
                overall_progress,
                Color::Magenta,
                buf,
            );
            y += 1;

            // Paper info line
            let info = format!(
                "  {} | {} | P:{:.0} | {}",
                paper.code, paper.source, paper.priority, paper.last_activity
            );
            let info_len = info.len().min(inner.width as usize);
            buf.set_string(
                inner.left(),
                y,
                &info[..info_len],
                Style::default().fg(Color::DarkGray),
            );
            y += 2;

            // Separator
            let sep: String = "=".repeat(inner.width as usize);
            buf.set_string(
                inner.left(),
                y,
                &sep,
                Style::default().fg(Color::Rgb(80, 40, 100)),
            );
            y += 1;

            // Get status for selected stage
            let prop_status = paper
                .stages
                .get(self.selected_stage)
                .map(|(_, st)| st.status.as_str());

            // Render detail for selected stage
            if let Some(stage) = d.stages.get(self.selected_stage) {
                let detail_area = Rect::new(
                    inner.left() + 1,
                    y,
                    inner.width.saturating_sub(2),
                    inner.bottom().saturating_sub(y),
                );
                self.render_stage_detail(stage, prop_status, detail_area, buf);
            }
        }
    }
}
