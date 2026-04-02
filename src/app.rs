use std::path::PathBuf;

/// Which screen the app is currently showing.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    ProjectSelect,
    InputFiles,
    PipelineExecution,
    Output,
}

/// A single input file slot.
#[derive(Debug, Clone, Default)]
pub struct InputSlot {
    pub label: &'static str,
    pub description: &'static str,
    pub files: Vec<PathBuf>,
    pub required: bool,
}

/// All the state for a project being configured / run.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub venue: String,
    pub slots: Vec<InputSlot>,
    pub output_dir: PathBuf,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let workspace = std::env::var("OPENCLAW_WORKSPACE")
            .unwrap_or_else(|_| r"C:\Users\Administrator\.openclaw\workspace".to_string());
        Self {
            name: String::new(),
            venue: String::new(),
            slots: vec![
                InputSlot {
                    label: "Context / Data",
                    description: "CSV, JSON, images, experiment logs -- the raw material",
                    files: Vec::new(),
                    required: true,
                },
                InputSlot {
                    label: "Direction / Novelty",
                    description: "Markdown or text describing the contribution and novelty angle",
                    files: Vec::new(),
                    required: true,
                },
                InputSlot {
                    label: "Target Paper / Template",
                    description: "LaTeX template or reference paper to match format/structure",
                    files: Vec::new(),
                    required: false,
                },
                InputSlot {
                    label: "Persona",
                    description: "TOML or text setting writing style, tone, and voice",
                    files: Vec::new(),
                    required: false,
                },
            ],
            output_dir: PathBuf::from(workspace).join("pipeline_output"),
        }
    }
}

/// A saved/recent project entry.
#[derive(Debug, Clone)]
pub struct RecentProject {
    pub name: String,
    pub venue: String,
    pub path: PathBuf,
    pub stages_done: u8,
    pub stages_total: u8,
    pub status: String,
    pub last_activity: String,
}

/// Top-level application state.
pub struct App {
    pub screen: Screen,
    pub config: ProjectConfig,
    pub recents: Vec<RecentProject>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let recents = Self::scan_recents();
        Self {
            screen: Screen::ProjectSelect,
            config: ProjectConfig::default(),
            recents,
            should_quit: false,
        }
    }

    /// Scan workspace for existing pipeline projects.
    fn scan_recents() -> Vec<RecentProject> {
        let workspace = std::env::var("OPENCLAW_WORKSPACE")
            .unwrap_or_else(|_| r"C:\Users\Administrator\.openclaw\workspace".to_string());

        // Read .paper_pipeline.json if it exists
        let json_path = std::path::Path::new(&workspace).join(".paper_pipeline.json");
        if let Ok(text) = std::fs::read_to_string(&json_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                let mut recents = Vec::new();
                if let Some(papers) = val.get("active_papers").and_then(|v| v.as_array()) {
                    for p in papers {
                        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let venue = p.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let last = p.get("last_activity").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let stage = p.get("stage").and_then(|v| v.as_str()).unwrap_or("");

                        let stages_done = p.get("stages").and_then(|v| v.as_object())
                            .map(|m| m.values().filter(|v| v.get("status").and_then(|s| s.as_str()) == Some("done")).count() as u8)
                            .unwrap_or(0);
                        let stages_total = p.get("stages").and_then(|v| v.as_object())
                            .map(|m| m.len() as u8)
                            .unwrap_or(8);

                        recents.push(RecentProject {
                            name,
                            venue,
                            path: json_path.clone(),
                            stages_done,
                            stages_total,
                            status: stage.to_string(),
                            last_activity: last,
                        });
                    }
                }
                return recents;
            }
        }
        Vec::new()
    }
}
