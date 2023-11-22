use crate::font::{DrawInst, FontHandle};
use image::{gif::Frame, Pixel, Rgb, Rgba, RgbaImage};

use rect_iter::RectRange;
use rogue_gym_core::dungeon::{Coord, X, Y};
use rogue_gym_core::error::{GameResult, ResultExt1};
use rogue_gym_uilib::Screen;
use rusttype::point;
use tuple_map::TupleMap2;

#[derive(Debug, Fail)]
#[fail(display = "EncodeError")]
pub struct EncodeError;

pub struct TermImage<'a: 'b, 'b> {
    buffer: RgbaImage,
    background: Rgba<u8>,
    fontcolor: Rgb<u8>,
    fontsize: u32,
    size: Coord,
    font: &'b mut FontHandle<'a>,
}

impl<'a, 'b> TermImage<'a, 'b> {
    pub fn new(
        width: X,
        height: Y,
        fontsize: u32,
        background: Rgb<u8>,
        fontcolor: Rgb<u8>,
        font: &'b mut FontHandle<'a>,
    ) -> Self {
        let (x, y) = (width.0, height.0).map(|x| x as u32 * (fontsize + 1));
        let mut buffer = RgbaImage::new(x / 2, y);
        let mut rgba = background.to_rgba();
        rgba.channels_mut()[3] = u8::max_value();
        buffer.pixels_mut().for_each(|p| *p = rgba);
        TermImage {
            buffer,
            fontcolor,
            background: rgba,
            fontsize,
            size: Coord::new(width, height),
            font,
        }
    }
    pub fn frame(&mut self) -> Frame {
        Frame::from_rgba(
            self.buffer.width() as u16,
            self.buffer.height() as u16,
            &mut *self.buffer,
        )
    }
    #[allow(unused)]
    pub fn save(&self, fname: &str) {
        self.buffer.save(fname).unwrap();
    }
}

impl<'a, 'b> Screen for TermImage<'a, 'b> {
    fn width(&self) -> X {
        self.size.x
    }
    fn height(&self) -> Y {
        self.size.y
    }
    fn clear_line(&mut self, row: Y) -> GameResult<()> {
        let ystart = row.0 as u32 * self.fontsize;
        let range = RectRange::from_ranges(0..self.buffer.width(), ystart..ystart + self.fontsize);
        let range = range
            .ok_or(EncodeError)
            .chain_err(|| "TermImage::clear_line Invalid range")?;
        for (x, y) in range {
            *self.buffer.get_pixel_mut(x, y) = self.background;
        }
        Ok(())
    }
    fn write_char(&mut self, pos: Coord, c: char) -> GameResult<()> {
        let (x, y) = (pos.x.0, pos.y.0).map(|x| x as u32 * self.fontsize);
        let mut rgba = self.fontcolor.to_rgba();
        let TermImage {
            ref mut buffer,
            background,
            ref mut font,
            ..
        } = self;
        font.draw(c, point(x / 2, y), |DrawInst { x, y, alpha }| {
            rgba.channels_mut()[3] = alpha;
            let cell = buffer.get_pixel_mut(x, y);
            *cell = *background;
            cell.blend(&rgba);
        });
        Ok(())
    }
}
