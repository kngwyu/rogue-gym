use image::Rgb;

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];
const SR_BASE1: [u8; 3] = [147, 161, 161];
const SR_BASE01: [u8; 3] = [88, 110, 117];
const SR_BASE3: [u8; 3] = [253, 246, 227];
const SR_BASE03: [u8; 3] = [0, 43, 54];

pub struct Theme {
    pub back: Rgb<u8>,
    pub font: Rgb<u8>,
}

impl Theme {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.replace('-', "_");
        Some(match &*s {
            "white" => Self::white(),
            "black" => Self::black(),
            "solarized_light" => Self::solarized_light(),
            "solarized_dark" => Self::solarized_dark(),
            x if x.starts_with("solarized") => Self::solarized_dark(),
            _ => return None,
        })
    }
    fn new(back: [u8; 3], font: [u8; 3]) -> Self {
        Theme {
            back: Rgb(back),
            font: Rgb(font),
        }
    }
    fn white() -> Self {
        Theme::new(WHITE, BLACK)
    }
    fn black() -> Self {
        Theme::new(BLACK, WHITE)
    }
    fn solarized_light() -> Self {
        Theme::new(SR_BASE3, SR_BASE01)
    }
    fn solarized_dark() -> Self {
        Theme::new(SR_BASE03, SR_BASE1)
    }
}
