use rect_iter::RectRange;
use super::Coord;
use error::GameResult;
use rng::{Rng, RngHandle};

pub(super) fn dig_maze<F>(range: RectRange<i32>, mut register: F, rng: &mut RngHandle)
where
    F: FnMut(Coord) -> GameResult<()>,
{

}
