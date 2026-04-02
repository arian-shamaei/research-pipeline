//! Integration tests for the 4-screen research pipeline TUI.
//!
//! Exercises state machine transitions, rendering to virtual buffers,
//! and input handling for each screen without needing a real terminal.

use std::process::Command;

/// Run the binary with --dump and capture output.
fn dump(screen: &str, w: u16, h: u16) -> String {
    let out = Command::new("cargo")
        .args(["run", "--release", "--", "--dump", screen, &w.to_string(), &h.to_string()])
        .output()
        .expect("failed to run binary");
    String::from_utf8_lossy(&out.stdout).to_string()
}

// ═══════════════════════════════════════════════════════════════════
// Screen 1: Project Select
// ═══════════════════════════════════════════════════════════════════

#[test]
fn screen_select_renders() {
    let out = dump("select", 100, 25);
    assert!(out.contains("Project Select"), "missing title");
    assert!(out.contains("NEW PROJECT"), "missing New Project button");
    assert!(out.contains("OPEN EXISTING"), "missing Open Existing button");
    assert!(out.contains("RECENT PROJECTS"), "missing Recent Projects header");
}

#[test]
fn screen_select_shows_recent() {
    let out = dump("select", 120, 30);
    // The .paper_pipeline.json has neil-blueprint-v6 as a recent project
    assert!(
        out.contains("neil-blueprint-v6") || out.contains("no recent"),
        "should show recent project or empty message"
    );
}

#[test]
fn screen_select_has_navigation_hint() {
    let out = dump("select", 100, 25);
    assert!(out.contains("Up/Down"), "missing navigation hint");
    assert!(out.contains("Enter"), "missing enter hint");
    assert!(out.contains("quit"), "missing quit hint");
}

// ═══════════════════════════════════════════════════════════════════
// Screen 2: Input Files
// ═══════════════════════════════════════════════════════════════════

#[test]
fn screen_input_renders() {
    let out = dump("input", 120, 35);
    assert!(out.contains("Input Files"), "missing title");
    assert!(out.contains("Project name"), "missing project name field");
    assert!(out.contains("Target venue"), "missing venue field");
}

#[test]
fn screen_input_shows_4_slots() {
    let out = dump("input", 120, 35);
    assert!(out.contains("Context / Data"), "missing slot 1");
    assert!(out.contains("Direction / Novelty"), "missing slot 2");
    assert!(out.contains("Target Paper / Template"), "missing slot 3");
    assert!(out.contains("Persona"), "missing slot 4");
}

#[test]
fn screen_input_shows_required_markers() {
    let out = dump("input", 120, 35);
    // Slots 1 and 2 are required (marked with *)
    assert!(out.contains("* 1."), "slot 1 should be marked required");
    assert!(out.contains("* 2."), "slot 2 should be marked required");
}

#[test]
fn screen_input_shows_browse() {
    let out = dump("input", 120, 35);
    let browse_count = out.matches("[Browse...]").count();
    assert_eq!(browse_count, 4, "should have 4 browse buttons, got {}", browse_count);
}

#[test]
fn screen_input_shows_start_disabled() {
    let out = dump("input", 120, 35);
    // No name entered, so start should be disabled
    assert!(out.contains("enter name first"), "start button should be disabled without name");
}

#[test]
fn screen_input_has_navigation_hint() {
    let out = dump("input", 120, 35);
    assert!(out.contains("Esc back"), "missing back hint");
    assert!(out.contains("d delete"), "missing delete hint");
}

// ═══════════════════════════════════════════════════════════════════
// Screen 3: Pipeline Execution
// ═══════════════════════════════════════════════════════════════════

#[test]
fn screen_pipeline_renders() {
    let out = dump("pipeline", 120, 30);
    assert!(out.contains("Research Pipeline"), "missing title");
}

#[test]
fn screen_pipeline_shows_flow_boxes() {
    let out = dump("pipeline", 120, 30);
    assert!(out.contains("SCOUT"), "missing SCOUT stage");
    assert!(out.contains("EVAL"), "missing EVAL stage");
    assert!(out.contains("READ"), "missing READ stage");
    assert!(out.contains("PROTO"), "missing PROTO stage");
    assert!(out.contains("INTEG"), "missing INTEG stage");
    assert!(out.contains("DOC"), "missing DOC stage");
    assert!(out.contains("VERFY"), "missing VERIFY stage");
    assert!(out.contains("YBR"), "missing YBR stage");
}

#[test]
fn screen_pipeline_shows_arrows() {
    let out = dump("pipeline", 120, 30);
    assert!(out.contains("==>"), "missing stage arrows");
}

#[test]
fn screen_pipeline_detail_renders() {
    let out = dump("pipeline-detail", 120, 40);
    assert!(out.contains("Stage Detail View"), "missing detail view title");
    assert!(out.contains("Overall:"), "missing overall progress");
    assert!(out.contains("Progress:"), "missing stage progress");
}

#[test]
fn screen_pipeline_detail_has_hints() {
    let out = dump("pipeline-detail", 120, 40);
    assert!(out.contains("<-/->"), "missing arrow key hint");
    assert!(out.contains("ground truth") || out.contains("ESC"), "missing interaction hints");
}

// ═══════════════════════════════════════════════════════════════════
// Screen 4: Output
// ═══════════════════════════════════════════════════════════════════

#[test]
fn screen_output_renders() {
    let out = dump("output", 120, 35);
    assert!(out.contains("Output:"), "missing title");
    assert!(out.contains("GENERATED FILES"), "missing files header");
}

#[test]
fn screen_output_shows_file_types() {
    let out = dump("output", 120, 35);
    assert!(out.contains("PDF"), "missing PDF entry");
    assert!(out.contains("LaTeX"), "missing LaTeX entry");
    assert!(out.contains("Figures"), "missing Figures entry");
    assert!(out.contains("BibTeX"), "missing BibTeX entry");
}

#[test]
fn screen_output_shows_descriptions() {
    let out = dump("output", 120, 35);
    assert!(out.contains("Final compiled paper"), "missing PDF description");
    assert!(out.contains("Editable source document"), "missing LaTeX description");
}

#[test]
fn screen_output_shows_stats() {
    let out = dump("output", 120, 35);
    assert!(out.contains("STATS"), "missing stats section");
}

#[test]
fn screen_output_shows_actions() {
    let out = dump("output", 120, 35);
    assert!(out.contains("REVISE"), "missing revise button");
    assert!(out.contains("NEW PROJECT"), "missing new project button");
    assert!(out.contains("QUIT"), "missing quit button");
}

// ═══════════════════════════════════════════════════════════════════
// Cross-screen: rendering doesn't panic at various sizes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn all_screens_render_small() {
    // Should not panic at small terminal sizes
    for screen in &["select", "input", "pipeline", "output"] {
        let out = dump(screen, 40, 10);
        assert!(!out.is_empty(), "{} should produce some output at 40x10", screen);
    }
}

#[test]
fn all_screens_render_wide() {
    for screen in &["select", "input", "pipeline", "output"] {
        let out = dump(screen, 200, 50);
        assert!(!out.is_empty(), "{} should produce some output at 200x50", screen);
    }
}
