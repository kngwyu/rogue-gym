//! General field representation
use super::{X, Y};
use num_traits::ToPrimitive;
use rect_iter::{Get2D, GetMut2D, IndexError};
use std::fmt;
use Tile;

/// Generic Cell trait
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Cell<S> {
    pub surface: S,
    pub attr: CellAttr,
}

impl<S> Cell<S> {
    /// if the cell is visible or not
    pub fn is_visible(&self) -> bool {
        self.attr.contains(CellAttr::IS_VISIBLE)
    }
}
impl<S> Cell<S> {
    pub fn with_default_attr(surface: S) -> Cell<S> {
        Cell {
            surface,
            attr: CellAttr::default(),
        }
    }
}

impl<S: Tile> Tile for Cell<S> {
    fn byte(&self) -> u8 {
        if self.is_visible() {
            self.surface.byte()
        } else {
            Self::NONE
        }
    }
}

bitflags! {
    #[derive(Serialize, Deserialize, Default)]
    pub struct CellAttr: u32 {
        /// the player has visited the cell
        const IS_VISITED = 0b00000001;
        /// the cell is hidden and the player needs to 's'
        const IS_HIDDEN  = 0b00000010;
        /// the cell is visible or not
        const IS_VISIBLE = 0b00000100;
        /// In many rogue like, draw status can be changed by the cell has been drawn or not.
        /// So to record the cell has been drawn or not is very important.
        const IS_DRAWN   = 0b00001000;
        /// the cell is locked
        const IS_LOCKED  = 0b00010000;
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field<S> {
    inner: Vec<Vec<Cell<S>>>,
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

impl<S: Tile> fmt::Display for Field<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for v in &self.inner {
            for cell in v {
                write!(f, "{}", cell.surface.byte() as char)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}
