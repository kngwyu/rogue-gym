//! General field representation
use num_traits::ToPrimitive;
use rect_iter::{Get2D, GetMut2D};

pub struct Cell<S> {
    surface: S,
    visited: bool,
    /// In many rogue-like, dungeon is veiled at first and then become
    /// apparent through exploring, so if the cell is already drawn is important.
    drawed: bool,
}

pub struct Field<S> {
    inner: Vec<Vec<Cell<S>>>,
}

impl<S> Get2D for Field<S> {
    type Item = Cell<S>;
    fn get_xy<T: ToPrimitive>(&self, x: T, y: T) -> Option<&Self::Item> {
        self.inner.get_xy(x, y)
    }
}

impl<S> GetMut2D for Field<S> {
    type Item = Cell<S>;
    fn get_mut_xy<T: ToPrimitive>(&mut self, x: T, y: T) -> Option<&mut Self::Item> {
        self.inner.get_mut_xy(x, y)
    }
}
