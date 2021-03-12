use tui::{
    style::{Color, Style},
    widgets::{Block, Widget},
};

#[derive(Default)]
pub struct Pie {
    pub style: Style,
    pub percent: Option<f32>,
}

const BLOCK_LIGHT: char = '\u{2591}';
const BLOCK_MEDIUM: char = '\u{2592}';
const BLOCK_DARK: char = '\u{2593}';
const BLOCK_FULL: char = '\u{2588}';

impl Widget for Pie {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let center = (area.width as f32 / 2., area.height as f32 / 2.);
        let radius = area.width.min(area.height) as f32 / 2. * 0.9; //0.9 is buffer
        let ratio = area.height as f32 / area.width as f32;
        let target_angle = 360. * self.percent.or(Some(1.)).unwrap();
        for y in 0..area.height {
            for x in 0..area.width {
                let distance = distance((x as f32, y as f32), center, ratio);
                let cell = buf.get_mut(area.left() + x, area.top() + y);
                let off = distance - radius;
                let angle_distance =
                     angle_between_points((x as f32, y as f32), center) - target_angle;
                match distance_to_char(off.max(angle_distance /3.0)) {
                    Some(c) => {
                        cell.set_char(c)
                            .set_fg(self.style.fg.or(Some(Color::Red)).unwrap());
                    }
                    None => {}
                }
            }
        }
    }
}

fn angle_between_points(center: (f32, f32), other: (f32, f32)) -> f32 {
    let (x, y) = other;
    let (cx, cy) = center;

    let radian = (y - cy).atan2(x - cx);
    let angle = radian * (180. / std::f32::consts::PI) - 90.;
    if angle < 0.0 {
        return angle + 360.0;
    }
    return angle;
}

fn distance_to_char(offset: f32) -> Option<char> {
    if offset < 0. {
        return Some(BLOCK_FULL);
    }
    if offset < 0.25 {
        return Some(BLOCK_DARK);
    }
    if offset < 0.5 {
        return Some(BLOCK_MEDIUM);
    }
    if offset < 0.8 {
        return Some(BLOCK_LIGHT);
    }
    return None;
}

fn distance(p1: (f32, f32), p2: (f32, f32), ratio: f32) -> f32 {
    //d=√((x_2-x_1)²+(y_2-y_1)²)
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let distance_squared = ((x2 - x1) * (x2 - x1) * ratio) + (y2 - y1) * (y2 - y1);
    return distance_squared.sqrt();
}
