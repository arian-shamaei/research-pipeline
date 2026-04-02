mod app;
mod pipeline;
mod screen_input;
mod screen_output;
mod screen_select;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::{App, Screen};
use pipeline::PaperPipelinePlane;
use screen_input::InputFilesState;
use screen_output::OutputState;
use screen_select::ProjectSelectState;

fn install_crash_reporter() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".into());
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic payload".into()
        };

        let report = format!(
            "=== RESEARCH PIPELINE CRASH REPORT ===\n\
             Location:  {}\n\
             Message:   {}\n\
             Version:   {}\n\
             === END CRASH REPORT ===\n",
            location,
            message,
            env!("CARGO_PKG_VERSION"),
        );

        let crash_path =
            r"C:\Users\Administrator\.openclaw\workspace\research_pipeline_crash.log";
        let _ = std::fs::write(crash_path, &report);
        eprintln!("{}", report);

        default_hook(info);
    }));
}

/// Render one frame to a virtual buffer and dump as text.
fn dump_frame(screen: &str, width: u16, height: u16) -> String {
    let mut app = App::new();
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);

    match screen {
        "select" => {
            let state = ProjectSelectState::new(&app);
            screen_select::render(area, &mut buf, &state, &app);
        }
        "input" => {
            let state = InputFilesState::new();
            screen_input::render(area, &mut buf, &state, &app);
        }
        "pipeline" => {
            let mut plane = PaperPipelinePlane::new();
            plane.update();
            plane.render(area, &mut buf);
        }
        "pipeline-detail" => {
            let mut plane = PaperPipelinePlane::new();
            plane.update();
            plane.render_detail(area, &mut buf);
        }
        "output" => {
            app.config.name = "test-paper".to_string();
            let state = OutputState::new();
            screen_output::render(area, &mut buf, &state, &app);
        }
        _ => {
            let state = ProjectSelectState::new(&app);
            screen_select::render(area, &mut buf, &state, &app);
        }
    }

    let mut output = String::new();
    for y in 0..height {
        for x in 0..width {
            let cell = &buf[(x, y)];
            output.push_str(cell.symbol());
        }
        output.push('\n');
    }
    output
}

fn main() -> anyhow::Result<()> {
    install_crash_reporter();

    let args: Vec<String> = std::env::args().collect();

    // --dump <screen> [width] [height]: render a screen to stdout
    if let Some(pos) = args.iter().position(|a| a == "--dump") {
        let screen = args.get(pos + 1).map(|s| s.as_str()).unwrap_or("select");
        let width: u16 = args
            .get(pos + 2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(120);
        let height: u16 = args
            .get(pos + 3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(40);

        print!("{}", dump_frame(screen, width, height));
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let mut app = App::new();

    // Per-screen state
    let mut select_state = ProjectSelectState::new(&app);
    let mut input_state = InputFilesState::new();
    let mut pipeline_plane = PaperPipelinePlane::new();
    let mut pipeline_detail = false;
    let mut output_state = OutputState::new();

    let tick_rate = Duration::from_millis(250);

    loop {
        if app.should_quit {
            return Ok(());
        }

        let frame_start = Instant::now();

        // Update (only pipeline screen ticks)
        if app.screen == Screen::PipelineExecution {
            pipeline_plane.update();
        }

        // Render
        terminal.draw(|frame| {
            let area = frame.area();
            match app.screen {
                Screen::ProjectSelect => {
                    screen_select::render(area, frame.buffer_mut(), &select_state, &app);
                }
                Screen::InputFiles => {
                    screen_input::render(area, frame.buffer_mut(), &input_state, &app);
                }
                Screen::PipelineExecution => {
                    if pipeline_detail {
                        pipeline_plane.render_detail(area, frame.buffer_mut());
                    } else {
                        pipeline_plane.render(area, frame.buffer_mut());
                    }
                }
                Screen::Output => {
                    screen_output::render(area, frame.buffer_mut(), &output_state, &app);
                }
            }
        })?;

        // Input
        let timeout = tick_rate.saturating_sub(frame_start.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Global quit
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    return Ok(());
                }

                match app.screen {
                    Screen::ProjectSelect => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up => select_state.move_up(),
                        KeyCode::Down => select_state.move_down(),
                        KeyCode::Enter => select_state.enter(&mut app),
                        _ => {}
                    },
                    Screen::InputFiles => match key.code {
                        KeyCode::Up => input_state.move_up(),
                        KeyCode::Down => input_state.move_down(),
                        KeyCode::Enter => input_state.enter(&mut app),
                        KeyCode::Esc => input_state.escape(&mut app),
                        KeyCode::Backspace => input_state.backspace(),
                        KeyCode::Char('d')
                            if !input_state.editing && !input_state.browsing =>
                        {
                            input_state.delete_file(&mut app)
                        }
                        KeyCode::Char(ch) if input_state.editing => {
                            input_state.type_char(ch)
                        }
                        _ => {}
                    },
                    Screen::PipelineExecution => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if pipeline_detail {
                                pipeline_detail = false;
                            } else {
                                app.screen = Screen::ProjectSelect;
                            }
                        }
                        KeyCode::Enter => {
                            if pipeline_detail {
                                pipeline_plane.handle_enter();
                            } else {
                                pipeline_detail = true;
                            }
                        }
                        KeyCode::Left if pipeline_detail => {
                            pipeline_plane.handle_arrow(false);
                        }
                        KeyCode::Right if pipeline_detail => {
                            pipeline_plane.handle_arrow(true);
                        }
                        KeyCode::Up => pipeline_plane.scroll_by(-1),
                        KeyCode::Down => pipeline_plane.scroll_by(1),
                        KeyCode::Char('d') => pipeline_detail = !pipeline_detail,
                        KeyCode::Char('o') => {
                            // Go to output screen
                            app.screen = Screen::Output;
                        }
                        _ => {}
                    },
                    Screen::Output => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Esc => {
                            app.screen = Screen::PipelineExecution;
                        }
                        KeyCode::Up => output_state.move_up(),
                        KeyCode::Down => output_state.move_down(),
                        KeyCode::Enter => output_state.enter(&mut app),
                        _ => {}
                    },
                }
            }
        }
    }
}
