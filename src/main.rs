mod pipeline;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use pipeline::PaperPipelinePlane;

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

fn main() -> anyhow::Result<()> {
    install_crash_reporter();

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
    let mut plane = PaperPipelinePlane::new();
    let mut detail_mode = false;
    let tick_rate = Duration::from_millis(250);

    loop {
        let frame_start = Instant::now();

        // Update
        plane.update();

        // Render
        terminal.draw(|frame| {
            let area = frame.area();

            if detail_mode {
                plane.render_detail(area, frame.buffer_mut());
            } else {
                plane.render(area, frame.buffer_mut());
            }
        })?;

        // Input
        let timeout = tick_rate.saturating_sub(frame_start.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if detail_mode {
                            detail_mode = false;
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        if detail_mode {
                            plane.handle_enter();
                        } else {
                            detail_mode = true;
                        }
                    }
                    KeyCode::Left => {
                        if detail_mode {
                            plane.handle_arrow(false);
                        }
                    }
                    KeyCode::Right => {
                        if detail_mode {
                            plane.handle_arrow(true);
                        }
                    }
                    KeyCode::Up => {
                        plane.scroll_by(-1);
                    }
                    KeyCode::Down => {
                        plane.scroll_by(1);
                    }
                    KeyCode::Char('d') => {
                        detail_mode = !detail_mode;
                    }
                    _ => {}
                }
            }
        }
    }
}
