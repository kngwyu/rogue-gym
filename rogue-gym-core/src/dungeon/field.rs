//! General field representation
use num_traits::ToPrimitive;
use rect_iter::{Get2D, GetMut2D};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use Drawable;

/// Surface trait(just alias)
pub trait Surface: Clone + Debug + Serialize + DeserializeOwned + Debug + Drawable {}

/// Generic Cell trait
pub struct Cell<S: Surface> {
    pub surface: S,
    pub attr: CellAttr,
}

impl<S: Surface> Cell<S> {
    /// if the cell is visible or not
    pub fn is_visible(&self) -> bool {
        self.attr.contains(CellAttr::IS_VISIBLE)
    }
}

impl<S: Surface> Drawable for Cell<S> {
    fn byte(&self) -> u8 {
        if self.is_visible() {
            self.surface.byte()
        } else {
            Self::NONE
        }
    }
}

bitflags! {
    #[derive(Serialize, Deserialize)]
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
    }
}

pub struct Field<S: Surface> {
    inner: Vec<Vec<Cell<S>>>,
}

impl<S: Surface> Get2D for Field<S> {
    type Item = Cell<S>;
    fn get_xy<T: ToPrimitive>(&self, x: T, y: T) -> Option<&Self::Item> {
        self.inner.get_xy(x, y)
    }
}

impl<S: Surface> GetMut2D for Field<S> {
    type Item = Cell<S>;
    fn get_mut_xy<T: ToPrimitive>(&mut self, x: T, y: T) -> Option<&mut Self::Item> {
        self.inner.get_mut_xy(x, y)
    }
}
