use crate::font::{DrawInst, FontHandle};
use image::{gif::Frame, Pixel, Rgb, Rgba, RgbaImage};
use rect_iter::{GetMut2D, RectRange};
use rogue_gym_core::dungeon::{Coord, X, Y};
use rusttype::point;
use tuple_map::TupleMap2;

pub struct TermImage {
    buffer: RgbaImage,
    background: Rgba<u8>,
    fontcolor: Rgb<u8>,
    fontsize: u32,
    size: Coord,
}

impl TermImage {
    pub fn new(
        width: X,
        height: Y,
        fontsize: u32,
        background: Rgb<u8>,
        fontcolor: Rgb<u8>,
    ) -> Self {
        let (x, y) = (width.0, height.0).map(|x| x as u32 * fontsize);
        let mut buffer = RgbaImage::new(x / 2, y);
        let mut rgba = background.to_rgba();
        rgba.channels_mut()[3] = u8::max_value();
        buffer.pixels_mut().for_each(|p| *p = rgba);
        TermImage {
            buffer,
            fontcolor,
            background: rgba,
            fontsize,
            size: Coord::new(width / 2, height),
        }
    }
    pub fn write_char<'a>(&mut self, pos: Coord, c: char, font: &mut FontHandle<'a>) {
        let (x, y) = (pos.x.0, pos.y.0).map(|x| x as u32 * self.fontsize);
        font.draw(c, point(x / 2, y), |DrawInst { x, y, alpha }| {
            let mut rgba = self.fontcolor.to_rgba();
            rgba.channels_mut()[3] = alpha;
            if let Ok(cell) = self.buffer.try_get_mut_xy(x, y) {
                *cell = rgba;
            }
        });
    }
    pub fn write_str<'a>(&mut self, start: Coord, s: &str, font: &mut FontHandle<'a>) {
        let mut current = start;
        for c in s.chars() {
            self.write_char(current, c, font);
            current.x.0 += 1;
        }
    }
    pub fn write_msg<'a>(&mut self, msg: &str, font: &mut FontHandle<'a>) {
        self.write_str(Coord::new(0, 0), msg, font)
    }
    pub fn write_status<'a>(&mut self, msg: &str, font: &mut FontHandle<'a>) {
        self.write_str(Coord::new(0, self.size.y - 1.into()), msg, font)
    }
    pub fn frame(&mut self) -> Frame {
        Frame::from_rgba(
            (self.size.x.0 as u32 * self.fontsize) as u16,
            (self.size.y.0 as u32 * self.fontsize) as u16,
            &mut *self.buffer,
        )
    }
    pub fn reset(&mut self) {
        for p in RectRange::from_ranges(
            0..self.buffer.width(),
            self.fontsize..self.buffer.height() - self.fontsize,
        )
        .unwrap()
        {
            *self.buffer.get_mut_p(p) = self.background;
        }
    }
}
