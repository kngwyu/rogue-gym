use rect_iter::RectRange;
use rusttype::{point, Font, FontCollection, Point, Scale};
use std::collections::{hash_map::Entry, HashMap};
use tuple_map::TupleMap2;

#[derive(Clone, Copy, Debug)]
pub struct DrawInst {
    pub x: u32,
    pub y: u32,
    pub alpha: u8,
}

pub struct FontHandle<'a> {
    font: Font<'a>,
    cache: HashMap<char, FontCache>,
    scale: u32,
    padding: u32,
}

impl<'a> FontHandle<'a> {
    pub fn new(font: &'a [u8], scale: u32) -> Self {
        let collection = FontCollection::from_bytes(font).expect("Invalid font data");
        let font = collection.into_font().expect("Invalid font data");
        Self {
            font,
            cache: HashMap::new(),
            scale,
            padding: scale / 4,
        }
    }
    pub fn draw<D>(&mut self, c: char, start: Point<u32>, mut draw_fn: D)
    where
        D: FnMut(DrawInst),
    {
        let scale = (self.scale / 2, self.scale);
        let start_t = (start.x, start.y);
        match self.cache.entry(c) {
            Entry::Occupied(entry) => {
                Self::draw_range(scale, entry.get(), start_t, &mut draw_fn);
            }
            Entry::Vacant(entry) => {
                let glyph = self.font.glyph(c);
                let glyph = glyph.scaled(Scale::uniform(self.scale as f32));
                let glyph = glyph.positioned(point(start.x as f32, start.y as f32));
                if let Some(bbox) = glyph.pixel_bounding_box() {
                    let offset = (bbox.min.x, bbox.min.y + scale.1 as i32).map(|x| x as u32);
                    let padding = self.padding;
                    let mut cache = FontCache::new(scale.1);
                    glyph.draw(|x, y, alpha| {
                        let (x, y) = (x, y).add(offset).sub(start_t);
                        let alpha = truncate_alpha(alpha);
                        if y >= padding {
                            cache.set(x, y - padding, alpha)
                        }
                    });
                    Self::draw_range(scale, &cache, start_t, &mut draw_fn);
                    entry.insert(cache);
                } else {
                    let cache = FontCache::new(scale.1);
                    Self::draw_range(scale, &cache, start_t, &mut draw_fn);
                    entry.insert(cache);
                }
            }
        }
    }
    fn draw_range<D>(scale: (u32, u32), cache: &FontCache, start: (u32, u32), draw_fn: &mut D)
    where
        D: FnMut(DrawInst),
    {
        let padding = scale.1 / 4;
        for (x, y) in RectRange::zero_start(scale.0, scale.1).unwrap() {
            let alpha = cache.get(x, y);
            let (x, y) = (x, y + padding).add(start);
            draw_fn(DrawInst { x, y, alpha });
        }
    }
}

fn truncate_alpha(f: f32) -> u8 {
    if f < 0.1 {
        0
    } else if f >= 1.0 {
        u8::max_value()
    } else {
        (f32::from(u8::max_value()) * f) as u8
    }
}

#[derive(Clone, Debug)]
struct FontCache {
    inner: Vec<u8>,
    scale: u32,
}

impl FontCache {
    fn new(scale: u32) -> FontCache {
        let len = (scale * scale) / 2;
        FontCache {
            inner: vec![0u8; len as usize],
            scale,
        }
    }
    fn set(&mut self, x: u32, y: u32, val: u8) {
        let idx = self.scale / 2 * y + x;
        self.inner[idx as usize] = val;
    }
    fn get(&self, x: u32, y: u32) -> u8 {
        let idx = self.scale / 2 * y + x;
        self.inner[idx as usize]
    }
}
