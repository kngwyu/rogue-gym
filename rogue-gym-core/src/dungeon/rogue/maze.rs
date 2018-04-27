use dungeon::{Coord, Direction};
use error::{GameResult, ResultExt};
use rect_iter::{Get2D, GetMut2D, RectRange};
use rng::RngHandle;
use std::collections::HashSet;
use tuple_map::TupleMap2;

pub(super) fn dig_maze<F>(
    range: RectRange<i32>,
    rng: &mut RngHandle,
    mut register: F,
) -> GameResult<()>
where
    F: FnMut(Coord) -> GameResult<()>,
{
    #[derive(Default, Clone)]
    struct Spot {
        exits: HashSet<Coord>,
        used: bool,
    }
    let (xlen, ylen) = (range.xlen(), range.ylen()).map(|i| i as usize);
    let mut spots = vec![vec![Spot::default(); xlen]; ylen];
    let start: Coord = range.upper_left().into();
    let mut dfs_stack = vec![start];
    while let Some(current_cd) = dfs_stack.pop() {
        register(current_cd).chain_err("[dungeon::rogue::maze::dig_maze]")?;
        let dig_dir = Direction::iter_variants()
            .take(4)
            .filter(|dir| {
                let nxt = current_cd + dir.to_cd().scale(2, 2);
                range.contains(nxt) && !spots.get_p(nxt).used
            })
            .enumerate()
            .find(|(i, _)| rng.does_happen(*i as u32 + 1))
            .map(|t| t.1);
        let dig_dir = match dig_dir {
            Some(d) => d,
            None => continue,
        };
        for cd in current_cd.endless_iter(dig_dir).skip(1).take(2) {
            register(cd).chain_err("[dungeon::rogue::maze::dig_maze]")?;
        }
    }
    Ok(())
}
