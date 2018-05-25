use dungeon::{Coord, Direction};
use error::{GameResult, ResultExt};
use fenwick::FenwickSet;
use rect_iter::RectRange;
use rng::RngHandle;
use std::collections::HashSet;

/// structure of maze
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Maze {
    pub(crate) range: RectRange<i32>,
    pub(crate) passages: FenwickSet,
}

impl Maze {
    /// return an iterator ehich enumerates passages
    pub(crate) fn passage_iter<'a>(&'a self) -> impl 'a + Iterator<Item = Coord> {
        self.passages
            .iter()
            .filter_map(move |u| self.range.nth(u).map(|t| Coord::from(t)))
    }
    /// jusges if a cd is in a maze
    pub(crate) fn has_cd(&self, cd: Coord) -> bool {
        let has_cd_impl = |cd| {
            let id = self.range.index(cd)?;
            Some(self.passages.contains(id))
        };
        has_cd_impl(cd) == Some(true)
    }
}

/// Dig into the maze in the specified range
/// range: a 2D range you want to dig the maze in
/// rng: random number generator
/// register: a closure which register the coordinates of maze into your dungeon
pub(super) fn dig_maze<F>(
    range: RectRange<i32>,
    rng: &mut RngHandle,
    mut register: F,
) -> GameResult<()>
where
    F: FnMut(Coord) -> GameResult<()>,
{
    let start: Coord = range.upper_left().into();
    register(start).chain_err(|| "dungeon::rogue::maze::dig_maze")?;
    let mut used = HashSet::new();
    used.insert(start);
    dig_impl(&range, rng, &mut register, &mut used, start)
        .chain_err(|| "dungeon::rogue::maze::dig_maze")
}

/// implementation of maze digging by DFS
// in this function we don't chain error, because this is sub function of dig_maze
fn dig_impl<F>(
    range: &RectRange<i32>,
    rng: &mut RngHandle,
    register: &mut F,
    used: &mut HashSet<Coord>,
    current_cd: Coord,
) -> GameResult<()>
where
    F: FnMut(Coord) -> GameResult<()>,
{
    loop {
        let dig_dir = Direction::iter_variants()
            .take(4)
            .filter(|dir| {
                let nxt = current_cd + dir.to_cd().scale(2, 2);
                range.contains(nxt) && !used.contains(&nxt)
            })
            .enumerate()
            .filter(|(i, _)| rng.does_happen(*i as u32 + 1))
            .last()
            .map(|t| t.1);
        let dig_dir = match dig_dir {
            Some(d) => d,
            None => break,
        };
        for cd in current_cd.direc_iter(dig_dir, |_| true).skip(1).take(2) {
            if used.insert(cd) {
                register(cd)?;
            }
        }
        let next = current_cd + dig_dir.to_cd().scale(2, 2);
        dig_impl(&range, rng, register, used, next)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use error::{ErrorId, ErrorKind};
    use rect_iter::GetMut2D;
    #[test]
    #[ignore]
    fn print_maze() {
        let mut rng = RngHandle::new();
        let range = RectRange::from_ranges(20..50, 10..20).unwrap();
        let mut buffer = vec![vec![false; 80]; 24];
        dig_maze(range.clone(), &mut rng, |cd| {
            if !range.contains(cd) {
                Err(ErrorId::MaybeBug.into_with(|| "dig_maze produced invalid Coordinate!"))
            } else {
                *buffer.get_mut_p(cd) = true;
                Ok(())
            }
        }).unwrap();
        for v in buffer {
            for f in v {
                if f {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
}
