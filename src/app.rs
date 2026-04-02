use std::path::PathBuf;

/// Root directory where all paper projects are stored.
pub fn papers_dir() -> PathBuf {
    let workspace = std::env::var("OPENCLAW_WORKSPACE")
        .unwrap_or_else(|_| r"C:\Users\Administrator\.openclaw\workspace".to_string());
    PathBuf::from(workspace).join("papers")
}

/// Which screen the app is currently showing.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    ProjectSelect,
    InputFiles,
    PipelineExecution,
    Output,
}

/// Whether a slot expects a directory or individual files.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlotKind {
    Directory,
    Files,
}

/// A single input file slot.
#[derive(Debug, Clone)]
pub struct InputSlot {
    pub label: &'static str,
    pub description: &'static str,
    pub kind: SlotKind,
    pub path: Option<PathBuf>,   // for Directory kind
    pub files: Vec<PathBuf>,     // for Files kind
    pub required: bool,
}

impl Default for InputSlot {
    fn default() -> Self {
        Self {
            label: "",
            description: "",
            kind: SlotKind::Files,
            path: None,
            files: Vec::new(),
            required: false,
        }
    }
}

/// All the state for a project being configured / run.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub venue: String,
    pub slots: Vec<InputSlot>,
    pub project_dir: PathBuf,
    pub output_dir: PathBuf,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            venue: String::new(),
            slots: vec![
                InputSlot {
                    label: "Context / Data",
                    description: "Directory containing CSV, JSON, images, experiment logs",
                    kind: SlotKind::Directory,
                    path: None,
                    files: Vec::new(),
                    required: true,
                },
                InputSlot {
                    label: "Direction / Novelty",
                    description: "Markdown or text describing the contribution and novelty angle",
                    kind: SlotKind::Files,
                    path: None,
                    files: Vec::new(),
                    required: true,
                },
                InputSlot {
                    label: "Target Paper / Template",
                    description: "LaTeX template or reference paper to match format/structure",
                    kind: SlotKind::Files,
                    path: None,
                    files: Vec::new(),
                    required: false,
                },
                InputSlot {
                    label: "Persona",
                    description: "TOML or text setting writing style, tone, and voice",
                    kind: SlotKind::Files,
                    path: None,
                    files: Vec::new(),
                    required: false,
                },
            ],
            project_dir: PathBuf::new(),
            output_dir: PathBuf::new(),
        }
    }
}

impl ProjectConfig {
    /// Set the project name and derive directory paths.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
        let slug = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        self.project_dir = papers_dir().join(&slug);
        self.output_dir = self.project_dir.join("output");
    }

    /// Create the project directory structure on disk.
    pub fn create_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.project_dir)?;
        std::fs::create_dir_all(self.project_dir.join("context"))?;
        std::fs::create_dir_all(self.project_dir.join("output"))?;
        std::fs::create_dir_all(self.project_dir.join("output").join("figures"))?;
        Ok(())
    }

    /// Save project manifest to project_dir/project.json.
    pub fn save_manifest(&self) -> std::io::Result<()> {
        let slot_data: Vec<serde_json::Value> = self
            .slots
            .iter()
            .map(|s| {
                let files: Vec<String> = s.files.iter()
                    .map(|f| f.to_string_lossy().to_string())
                    .collect();
                serde_json::json!({
                    "label": s.label,
                    "kind": if s.kind == SlotKind::Directory { "directory" } else { "files" },
                    "path": s.path.as_ref().map(|p| p.to_string_lossy().to_string()),
                    "files": files,
                    "required": s.required,
                })
            })
            .collect();

        let manifest = serde_json::json!({
            "name": self.name,
            "venue": self.venue,
            "project_dir": self.project_dir.to_string_lossy(),
            "output_dir": self.output_dir.to_string_lossy(),
            "slots": slot_data,
            "created": chrono::Local::now().to_rfc3339(),
        });

        let path = self.project_dir.join("project.json");
        let text = serde_json::to_string_pretty(&manifest)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, text)
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
        // Ensure papers/ directory exists
        let _ = std::fs::create_dir_all(papers_dir());

        let recents = Self::scan_recents();
        Self {
            screen: Screen::ProjectSelect,
            config: ProjectConfig::default(),
            recents,
            should_quit: false,
        }
    }

    /// Scan papers/ directory for saved projects.
    pub fn scan_recents() -> Vec<RecentProject> {
        let dir = papers_dir();
        let mut recents = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let manifest_path = path.join("project.json");
                if let Ok(text) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        let name = val.get("name").and_then(|v| v.as_str())
                            .unwrap_or("").to_string();
                        let venue = val.get("venue").and_then(|v| v.as_str())
                            .unwrap_or("").to_string();
                        let created = val.get("created").and_then(|v| v.as_str())
                            .unwrap_or("").to_string();

                        // Check for pipeline state
                        let state_path = path.join("pipeline_state.json");
                        let (stages_done, stages_total, status) =
                            if let Ok(st) = std::fs::read_to_string(&state_path) {
                                if let Ok(sv) = serde_json::from_str::<serde_json::Value>(&st) {
                                    let done = sv.get("stages_done")
                                        .and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                                    let total = sv.get("stages_total")
                                        .and_then(|v| v.as_u64()).unwrap_or(8) as u8;
                                    let s = sv.get("status")
                                        .and_then(|v| v.as_str()).unwrap_or("new").to_string();
                                    (done, total, s)
                                } else {
                                    (0, 8, "new".to_string())
                                }
                            } else {
                                (0, 8, "new".to_string())
                            };

                        recents.push(RecentProject {
                            name,
                            venue,
                            path: path.clone(),
                            stages_done,
                            stages_total,
                            status,
                            last_activity: created,
                        });
                    }
                }
            }
        }

        // Sort by last_activity descending
        recents.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        recents
    }

    /// Reload the recent projects list.
    pub fn refresh_recents(&mut self) {
        self.recents = Self::scan_recents();
    }

    /// Load a recent project into config.
    pub fn load_project(&mut self, recent: &RecentProject) {
        let manifest_path = recent.path.join("project.json");
        if let Ok(text) = std::fs::read_to_string(&manifest_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                self.config.name = val.get("name").and_then(|v| v.as_str())
                    .unwrap_or("").to_string();
                self.config.venue = val.get("venue").and_then(|v| v.as_str())
                    .unwrap_or("").to_string();
                self.config.project_dir = recent.path.clone();
                self.config.output_dir = recent.path.join("output");

                // Restore slot data
                if let Some(slots) = val.get("slots").and_then(|v| v.as_array()) {
                    for (i, sv) in slots.iter().enumerate() {
                        if let Some(slot) = self.config.slots.get_mut(i) {
                            slot.path = sv.get("path")
                                .and_then(|v| v.as_str())
                                .map(PathBuf::from);
                            slot.files = sv.get("files")
                                .and_then(|v| v.as_array())
                                .map(|a| a.iter()
                                    .filter_map(|f| f.as_str().map(PathBuf::from))
                                    .collect())
                                .unwrap_or_default();
                        }
                    }
                }
            }
        }
    }
}
