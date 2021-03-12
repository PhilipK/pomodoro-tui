mod image_widget;
mod pie;
use image::io::Reader as ImageReader;
use pie::Pie;
use std::io;
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

use std::thread::sleep;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let total_time_seconds = 60. * 25.;
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
            pie.style.fg = Some(Color::Blue);
            f.render_widget(pie, chunks[0]);
            f.render_widget(span, chunks[1]);            
        })?;
        sleep(Duration::new(0, 16000000)); //about 60 fps
    }
    terminal.clear()?;

    Ok(())
}
