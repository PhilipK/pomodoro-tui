mod pie;
use pie::Pie;
use std::io;
use std::path::Path;
use tui::layout::Alignment;
use tui::Terminal;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Color,
    text,
    widgets::Paragraph,
};

use notify_rust::Notification;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use rodio::Source;
use std::fs::File;
use std::io::BufReader;

fn get_text_color(workmode: Phase, minutes_remaining: i32) -> Option<Color> {
    match (workmode, minutes_remaining) {
        (Phase::Work, x) if x <= 4 => Some(Color::Red),
        (Phase::Break, x) if x <= 0 => Some(Color::Green),
        (Phase::LongBreak, x) if x <= 0 => Some(Color::Blue),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_text_color_none() {
        assert_eq!(get_text_color(Phase::Break, 20), None);
    }

    #[test]
    fn test_get_text_color_red() {
        assert_eq!(get_text_color(Phase::Break, 4), Some(Color::Red));
    }

    #[test]
    fn test_get_text_color_none_break() {
        assert_eq!(get_text_color(Phase::Work, 1), None);
    }

    #[test]
    fn test_get_text_color_green() {
        assert_eq!(get_text_color(Phase::Work, 0), Some(Color::Green));
    }
}

const STORAGE_LOCATION: &'static str = "storage";

#[derive(Clone, Copy)]
enum Phase {
    Work,
    Break,
    LongBreak,
}

impl Phase {
    pub fn name(&self) -> &str {
        match self {
            Phase::Work => "work",
            Phase::Break => "break",
            Phase::LongBreak => "long break",
        }
    }
}

fn get_todays_progress() -> Vec<Phase> {
    let path = progress_file_path();
    if path.exists() {
        let content = std::fs::read_to_string(path).unwrap();
        let mut res = Vec::with_capacity(content.len());
        for char in content.chars() {
            res.push(match char {
                '0' => Phase::Work,
                '1' => Phase::Break,
                '2' => Phase::LongBreak,
                _ => panic!("Unknown char type"),
            });
        }
        res
    } else {
        vec![]
    }
}

fn get_next_phase(progress: &Vec<Phase>) -> Phase {
    let last = progress.last();
    match last {
        Some(Phase::Work) => {
            let mut works_since_long_break = 0;
            for cur_phase in progress.iter().rev() {
                match cur_phase {
                    Phase::Work => works_since_long_break += 1,
                    Phase::Break => (),
                    Phase::LongBreak => break,
                }
            }
            if works_since_long_break >= 4 {
                Phase::LongBreak
            } else {
                Phase::Break
            }
        }
        Some(Phase::Break | &Phase::LongBreak) | None => Phase::Work,
    }
}

fn progress_file_path() -> std::path::PathBuf {
    let now = chrono::Utc::now();
    let file_name = format!("{}", now.date());
    let file_name = file_name.strip_suffix("UTC").unwrap_or(file_name.as_str());
    let path = Path::new(STORAGE_LOCATION).join(file_name);
    path
}

fn save_progress(phases: Vec<Phase>) {
    let path = progress_file_path();
    std::fs::create_dir_all(Path::new(STORAGE_LOCATION)).unwrap();
    let content: Vec<u8> = phases
        .iter()
        .map(|phase| match phase {
            Phase::Work => b'0',
            Phase::Break => b'1',
            Phase::LongBreak => b'2',
        })
        .collect();
    std::fs::write(path, &content.as_slice()).unwrap()
}

fn main() -> Result<(), io::Error> {
    let mut todays_progress = get_todays_progress();
    let phase = get_next_phase(&todays_progress);

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let total_time_seconds = 60.
        * (match phase {
            Phase::Work => 25.,
            Phase::Break => 5.,
            Phase::LongBreak => 15.,
        });

    let mut played_warning = false;
    let now = SystemTime::now();
    while now.elapsed().unwrap().as_secs_f32() < total_time_seconds {
        terminal.draw(|f| {
            let elapsed_seconds = now.elapsed().unwrap().as_secs_f32();

            let remaining_seconds = total_time_seconds - elapsed_seconds;
            let percent = elapsed_seconds / total_time_seconds;

            let progress_texts: Vec<text::Span> = (&todays_progress)
                .iter()
                .map(|phase| {
                    let short_name = match phase {
                        Phase::Work => "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}",
                        Phase::Break => "\u{2588}",
                        Phase::LongBreak => "\u{2588}\u{2588}\u{2588}",
                    };

                    let mut text = text::Span::raw(short_name);
                    text.style.fg = Some(match phase {
                        Phase::Work => Color::Red,
                        Phase::Break => Color::Green,
                        Phase::LongBreak => Color::Blue,
                    });
                    text
                })
                .collect();
            let progress_spans = text::Spans::from(progress_texts);
            let progress_paragraph = Paragraph::new(progress_spans).alignment(Alignment::Center);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(90),
                        Constraint::Percentage(5),
                        Constraint::Percentage(5),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let minutes_remaining = (remaining_seconds / 60.) as i32;
            let seconds_remaining = remaining_seconds as i32 % 60;
            let mut time_text =
                text::Span::raw(format!("{:02}:{:02}", minutes_remaining, seconds_remaining));
            time_text.style.fg = get_text_color(phase, minutes_remaining);

            let time_paragraph = Paragraph::new(time_text).alignment(Alignment::Center);

            let mut pie = Pie {
                percent: Some(percent),
                ..Pie::default()
            };
            pie.style.fg = Some(match phase {
                Phase::Work => Color::Red,
                Phase::Break => Color::Green,
                Phase::LongBreak => Color::Blue,
            });
            f.render_widget(pie, chunks[0]);
            f.render_widget(time_paragraph, chunks[1]);
            f.render_widget(progress_paragraph, chunks[2]);
            if minutes_remaining < 1 && !played_warning {
                if let Ok(file) = File::open("warning.mp3") {
                    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                    stream_handle.play_raw(source.convert_samples()).unwrap();
                    sleep(Duration::new(2, 0));
                };
                played_warning = true;
            }
        })?;
        sleep(Duration::from_secs_f32(0.3)); //Update once a second
    }
    terminal.clear()?;

    Notification::new()
        .summary("Pomodoro done")
        .body(format!("Your {} is over", phase.name()).as_str())
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
    todays_progress.push(phase);
    save_progress(todays_progress);

    Ok(())
}
