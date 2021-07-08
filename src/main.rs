mod pie;
use pie::Pie;
use std::{env, io};
use tui::layout::Alignment;
use tui::Terminal;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Color,
    text::{self},
    widgets::Paragraph,
};

use notify_rust::Notification;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use rodio::Source;
use std::fs::File;
use std::io::BufReader;

fn get_text_color(workmode: bool, minutes_remaining: i32) -> Option<Color> {
    match (workmode, minutes_remaining) {
        (true, x) if x <= 4 => Some(Color::Red),
        (false, x) if x <= 0 => Some(Color::Green),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_text_color_none() {
        assert_eq!(get_text_color(true, 20), None);
    }

    #[test]
    fn test_get_text_color_red() {
        assert_eq!(get_text_color(true, 4), Some(Color::Red));
    }

    #[test]
    fn test_get_text_color_none_break() {
        assert_eq!(get_text_color(false, 1), None);
    }

    #[test]
    fn test_get_text_color_green() {
        assert_eq!(get_text_color(false, 0), Some(Color::Green));
    }
}

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

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
                .split(f.size());

            let minutes_remaining = (remaining_seconds / 60.) as i32;
            let seconds_remaining = remaining_seconds as i32 % 60;
            let mut time_text =
                text::Span::raw(format!("{}:{}", minutes_remaining, seconds_remaining));

            time_text.style.fg = get_text_color(workmode, minutes_remaining);

            let span = Paragraph::new(time_text).alignment(Alignment::Center);

            let mut pie = Pie {
                percent: Some(percent),
                ..Pie::default()
            };
            pie.style.fg = Some(if workmode {
                Color::Red
            } else {
                Color::LightGreen
            });
            f.render_widget(pie, chunks[0]);
            f.render_widget(span, chunks[1]);
        })?;
        sleep(Duration::new(1, 0)); //Update once a second
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
    if let Ok(file) = File::open("ding.mp3") {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        stream_handle.play_raw(source.convert_samples()).unwrap();
        sleep(Duration::new(2, 0));
    };

    Ok(())
}
