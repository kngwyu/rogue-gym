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
}

impl<'a> FontHandle<'a> {
    pub fn new(font: &'a [u8], scale: u32) -> Self {
        let collection = FontCollection::from_bytes(font).expect("Invalid font data");
        let font = collection.into_font().expect("Invalid font data");
        Self {
            font,
            cache: HashMap::new(),
            scale,
        }
    }
    pub fn draw<D>(&mut self, c: char, start: Point<u32>, mut draw_fn: D)
    where
        D: FnMut(DrawInst),
    {
        let scale = (self.scale / 2, self.scale);
        let start_t = (start.x, start.y);
        let is_valid = move |x, y| {
            let p = start_t;
            x >= p.0 && y >= p.1 && x < p.0 + scale.0 && y < p.1 + scale.1
        };
        match self.cache.entry(c) {
            Entry::Occupied(entry) => {
                for (x, y) in RectRange::zero_start(scale.0, scale.1).unwrap() {
                    let alpha = entry.get().get(x, y);
                    let (x, y) = (x, y).add(start_t);
                    draw_fn(DrawInst { x, y, alpha });
                }
            }
            Entry::Vacant(entry) => {
                let glyph = self.font.glyph(c);
                let glyph = glyph.scaled(Scale::uniform(self.scale as f32));
                let glyph = glyph.positioned(point(start.x as f32, start.y as f32));
                let mut cache = FontCache::new(scale.1);
                if let Some(bbox) = glyph.pixel_bounding_box() {
                    let offset = (bbox.min.x, bbox.max.y).map(|x| x as u32);
                    glyph.draw(|x, y, alpha| {
                        let (x, y) = (x, y).add(offset);
                        if is_valid(x, y) {
                            let alpha = truncate_alpha(alpha);
                            let (x, y) = (x, y).map(|x| x as u32);
                            draw_fn(DrawInst { x, y, alpha });
                            let (x, y) = (x, y).sub(start_t);
                            cache.set(x, y, alpha)
                        }
                    });
                }
                entry.insert(cache);
            }
        }
    }
}

fn truncate_alpha(f: f32) -> u8 {
    //let f = f + 0.2;
    if f > 1.0 {
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
