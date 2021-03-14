mod image_widget;
mod pie;
use image::io::Reader as ImageReader;
use pie::Pie;
use std::{env, io};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{self, Text},
    widgets::Paragraph,
};
use tui::{
    widgets::{Block, Borders},
    Terminal,
};

use notify_rust::Notification;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use rodio::Source;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let mut workmode = true;
    if args.len() >= 2 && &args[1] == "break" {
        workmode = false;
    }

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let total_time_seconds = 60. * (if workmode { 25. } else { 5. });
    let now = SystemTime::now();
    while now.elapsed().unwrap().as_secs_f32() < total_time_seconds {
        terminal.draw(|f| {
            let elapsed_seconds = now.elapsed().unwrap().as_secs_f32();
            let remaining_seconds = total_time_seconds - elapsed_seconds;
            let percent = elapsed_seconds / total_time_seconds;
            let minutes_remaining = (remaining_seconds / 60.) as i32;
            let seconds_remaining = remaining_seconds as i32 % 60;

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(f.size());
            let span = Paragraph::new(text::Span::raw(format!(
                "{}:{}",
                minutes_remaining, seconds_remaining
            )));

            let mut pie = Pie::default();
            pie.percent = Some(percent);
            pie.style.fg = Some(if workmode {
                Color::Red
            } else {
                Color::LightGreen
            });
            f.render_widget(pie, chunks[0]);
            f.render_widget(span, chunks[1]);
        })?;
        sleep(Duration::new(0, 16000000)); //about 60 fps
    }
    terminal.clear()?;

    Notification::new()
        .summary("Pomodoro done")
        .body(format!("Your {} is over", if workmode { "work" } else { "break" }).as_str())
        .icon("pomodoro-tui")
        .appname("pomodoro-tui")
        .timeout(0) // this however is
        .show()
        .unwrap();

    // Load a sound from a file, using a path relative to Cargo.toml
    match File::open("ding.mp3") {
        Ok(file) => {
            let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
            stream_handle.play_raw(source.convert_samples()).unwrap();
            sleep(Duration::new(2, 0)); //about 60 fps
        }
        Err(_) => {}
    };
    
    Ok(())
}
