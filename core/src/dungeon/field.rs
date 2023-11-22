//! General field representation
use super::{X, Y};
use crate::tile::{Drawable, Tile};
use num_traits::ToPrimitive;
use rect_iter::{Get2D, GetMut2D, IndexError, RectRange};
use serde::{Deserialize, Serialize};
use std::fmt;
use tuple_map::TupleMap2;

/// Generic representation of Cell
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Cell<S> {
    pub surface: S,
    pub attr: CellAttr,
}

impl<S> Cell<S> {
    /// now player is near the cell
    #[inline]
    pub fn approached(&mut self) {
        if self.attr.contains(CellAttr::IS_HIDDEN) {
            return;
        }
        self.attr |= CellAttr::HAS_DRAWN;
        self.visible(true);
    }

    /// now player leaves neighbor cell
    #[inline]
    pub fn left(&mut self) {
        if self.attr.contains(CellAttr::IS_DARK) {
            self.visible(false);
        }
    }

    /// if the object on the cell is visible or not
    #[inline]
    pub fn is_obj_visible(&self) -> bool {
        self.attr.contains(CellAttr::IS_VISIBLE) || self.attr.contains(CellAttr::HAS_DRAWN)
    }

    /// if the cell is visible or not
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.attr.contains(CellAttr::IS_VISIBLE)
    }

    /// change the visibility of the cell
    #[inline]
    pub fn visible(&mut self, on: bool) {
        if on {
            self.attr.insert(CellAttr::IS_VISIBLE);
        } else {
            self.attr.remove(CellAttr::IS_VISIBLE);
        }
    }

    #[inline]
    pub fn visit(&mut self) {
        self.attr.insert(CellAttr::IS_VISITED);
    }

    #[inline]
    pub fn is_visited(&self) -> bool {
        self.attr.contains(CellAttr::IS_VISITED)
    }

    /// construct a cell with default attribute
    #[inline]
    pub fn with_default_attr(surface: S) -> Cell<S> {
        Cell {
            surface,
            attr: CellAttr::default(),
        }
    }
    #[inline]
    pub fn is_hidden(&self) -> bool {
        self.attr.contains(CellAttr::IS_HIDDEN)
    }
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.attr.contains(CellAttr::IS_LOCKED)
    }
    #[inline]
    pub fn unlock(&mut self) {
        self.attr.remove(CellAttr::IS_LOCKED | CellAttr::IS_HIDDEN);
        self.visible(true)
    }
}

impl<S: Drawable> Drawable for Cell<S> {
    fn tile(&self) -> Tile {
        if self.is_visible() {
            self.surface.tile()
        } else {
            Self::NONE
        }
    }

    const NONE: Tile = Tile(b' ');

    fn color(&self) -> crate::tile::Color {
        crate::tile::Color(0)
    }
}

bitflags! {
    #[derive(Serialize, Deserialize, Default)]
    pub struct CellAttr: u32 {
        /// the player has visited the cell
        const IS_VISITED = 0b00_000_001;
        /// the cell is hidden and the player needs to 's'
        const IS_HIDDEN  = 0b00_000_010;
        /// the cell is visible or not
        const IS_VISIBLE = 0b00_000_100;
        /// In many rogue like, draw status can be changed by the cell has been drawn or not.
        /// So to record the cell has been drawn or not is very important.
        const HAS_DRAWN  = 0b00_001_000;
        /// the cell is locked
        const IS_LOCKED  = 0b00_010_000;
        /// the cell is in dark room
        const IS_DARK    = 0b00_100_000;
    }
}

/// generic representation of Field
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field<S> {
    inner: Vec<Cell<S>>,
    width: X,
    height: Y,
}

impl<S> Field<S> {
    pub fn size(&self) -> RectRange<i32> {
        RectRange::zero_start(self.width.0, self.height.0)
            .expect("[Field::size] Invalid dungeon size")
    }
    pub fn width(&self) -> X {
        self.width
    }
    pub fn height(&self) -> Y {
        self.height
    }
    pub fn size_ytrimed(&self) -> Option<RectRange<i32>> {
        RectRange::from_corners((0, 1), (self.width.0, self.height.0 - 1))
    }
}

impl<S: Clone> Field<S> {
    pub fn new(width: X, height: Y, init: Cell<S>) -> Self {
        let (w, h) = (width.0 as usize, height.0 as usize);
        Field {
            inner: vec![init; w * h],
            width,
            height,
        }
    }
}

impl<S> Get2D for Field<S> {
    type Item = Cell<S>;
    fn try_get_xy<T: ToPrimitive>(&self, x: T, y: T) -> Result<&Self::Item, IndexError> {
        let (x, y) = (x, y).map(|n| n.to_i64().unwrap());
        let x = x.to_usize().ok_or(IndexError::X(x))?;
        let y = y.to_usize().ok_or(IndexError::Y(y))?;
        if x > self.width.0 as usize {
            return Err(IndexError::X(x as i64));
        }
        if y > self.height.0 as usize {
            return Err(IndexError::Y(y as i64));
        }
        let id = y * self.width.0 as usize + x;
        Ok(&self.inner[id])
    }
}

impl<S> GetMut2D for Field<S> {
    fn try_get_mut_xy<T: ToPrimitive>(
        &mut self,
        x: T,
        y: T,
    ) -> Result<&mut Self::Item, IndexError> {
        let (x, y) = (x, y).map(|n| n.to_i64().unwrap());
        let x = x.to_usize().ok_or(IndexError::X(x))?;
        let y = y.to_usize().ok_or(IndexError::Y(y))?;
        if x > self.width.0 as usize {
            return Err(IndexError::X(x as i64));
        }
        if y > self.height.0 as usize {
            return Err(IndexError::Y(y as i64));
        }
        let id = y * self.width.0 as usize + x;
        Ok(&mut self.inner[id])
    }
}

impl<S: Drawable> fmt::Display for Field<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height.0 as usize {
            for x in 0..self.width.0 as usize {
                let cell = &self.inner[y * self.width.0 as usize + x];
                write!(f, "{}", cell.surface.tile())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
