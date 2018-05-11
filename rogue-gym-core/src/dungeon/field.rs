//! General field representation
use super::{X, Y};
use num_traits::ToPrimitive;
use rect_iter::{Get2D, GetMut2D, IndexError, RectRange};
use std::fmt;
use tile::{Drawable, Tile};

/// Generic representation of Cell
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Cell<S> {
    pub surface: S,
    pub attr: CellAttr,
}

impl<S> Cell<S> {
    pub fn approached_by_player(&mut self) {
        if self.attr.contains(CellAttr::IS_LOCKED) || self.attr.contains(CellAttr::IS_HIDDEN) {
            return;
        }
        self.attr |= CellAttr::IS_VISIBLE;
    }
    /// if the cell is visible or not
    pub fn is_visible(&self) -> bool {
        self.attr.contains(CellAttr::IS_VISIBLE)
    }
    pub fn visit(&mut self) {
        self.attr |= CellAttr::IS_VISITED;
    }
    /// construct a cell with default attribute
    pub fn with_default_attr(surface: S) -> Cell<S> {
        Cell {
            surface,
            attr: CellAttr::default(),
        }
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
        const IS_DRAWN   = 0b00_001_000;
        /// the cell is locked
        const IS_LOCKED  = 0b00_010_000;
    }
}

impl CellAttr {
    pub fn is_hidden(&self) -> bool {
        self.contains(CellAttr::IS_HIDDEN) || self.contains(CellAttr::IS_LOCKED)
    }
}

/// generic representation of Field
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field<S> {
    inner: Vec<Vec<Cell<S>>>,
}

impl<S> Field<S> {
    pub fn size(&self) -> Option<RectRange<i32>> {
        let y = self.inner.len();
        let x = self.inner.get(0)?.len();
        RectRange::zero_start(x as i32, y as i32)
    }
}

impl<S: Clone> Field<S> {
    pub fn new(width: X, height: Y, init: Cell<S>) -> Self {
        let (w, h) = (width.0 as usize, height.0 as usize);
        Field {
            inner: vec![vec![init; w]; h],
        }
    }
}

impl<S> Get2D for Field<S> {
    type Item = Cell<S>;
    fn try_get_xy<T: ToPrimitive>(&self, x: T, y: T) -> Result<&Self::Item, IndexError> {
        self.inner.try_get_xy(x, y)
    }
}

impl<S> GetMut2D for Field<S> {
    fn try_get_mut_xy<T: ToPrimitive>(
        &mut self,
        x: T,
        y: T,
    ) -> Result<&mut Self::Item, IndexError> {
        self.inner.try_get_mut_xy(x, y)
    }
}

impl<S: Drawable> fmt::Display for Field<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for v in &self.inner {
            for cell in v {
                write!(f, "{}", cell.surface.tile())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
