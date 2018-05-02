use super::{Room, RoomKind};
use dungeon::{Coord, Direction, X, Y};
use error::{GameResult, ResultExt};
use fenwick::FenwickSet;
use fixedbitset::FixedBitSet;
use rect_iter::{IntoTuple2, RectRange};
use rng::{Rng, RngHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use tuple_map::TupleMap2;

/// make passages between rooms
pub(crate) fn dig_passges<F>(
    rooms: &[Room],
    xrooms: X,
    yrooms: Y,
    rng: &mut RngHandle,
    mut register: F,
) -> GameResult<()>
where
    F: FnMut(Coord) -> GameResult<()>,
{
    let range = RectRange::zero_start(xrooms.0, yrooms.0).unwrap();
    let mut graph: Vec<_> = range
        .into_iter()
        .enumerate()
        .map(|(i, t)| Node::new(xrooms, yrooms, t, i))
        .collect();
    let num_rooms = rooms.len();
    let mut selected = FenwickSet::with_capacity(num_rooms);
    let mut cur_room = rng.range(0..num_rooms);
    selected.insert(cur_room);
    while selected.len() < num_rooms {
        let nxt = (0..num_rooms)
            .filter_map(|i| {
                if selected.contains(i) {
                    return None;
                }
                graph[cur_room].candidates.get(&i).map(|dir| (i, *dir))
            })
            .enumerate()
            .filter(|(i, _)| rng.does_happen(*i as u32 + 1))
            .last()
            .map(|t| t.1);
        if let Some((nxt_room, direction)) = nxt {
            selected.insert(nxt_room);
            connect_2rooms(
                &rooms[cur_room],
                &rooms[nxt_room],
                direction,
                rng,
                &mut register,
            ).chain_err("[passages::dig_passges]")?;
        } else {
            cur_room = selected.select(rng).unwrap();
        }
    }
    Ok(())
}

fn connect_2rooms<F>(
    room1: &Room,
    room2: &Room,
    direction: Direction,
    rng: &mut RngHandle,
    register: &mut F,
) -> GameResult<()>
where
    F: FnMut(Coord) -> GameResult<()>,
{
    let start = match room1.kind {
        RoomKind::Normal { ref range } => {
            let candidates = inclusive_edges(range, direction);
            *rng.choose(&candidates).unwrap()
        }
        RoomKind::Maze { ref range } => unimplemented!(),
        RoomKind::Empty { up_left } => up_left,
    };
    Ok(())
}

fn inclusive_edges(range: &RectRange<i32>, direction: Direction) -> Vec<Coord> {
    let bound_x = X(range.get_x().end - 1);
    let bound_y = Y(range.get_y().end - 1);
    match direction {
        Direction::Down => {
            let start: Coord = range.lower_left().into();
            start
                .slide_x(1)
                .direc_iter(Direction::Right, |cd| cd.x >= bound_x)
                .collect()
        }
        Direction::Left => {
            let start: Coord = range.upper_left().into();
            start
                .slide_y(1)
                .direc_iter(Direction::Down, |cd| cd.y >= bound_y)
                .collect()
        }
        Direction::Right => {
            let start: Coord = range.upper_right().into();
            println!("{:?}", start);
            start
                .slide_y(1)
                .direc_iter(Direction::Down, |cd| cd.y >= bound_y)
                .collect()
        }
        Direction::Up => {
            let start: Coord = range.upper_left().into();
            start
                .slide_x(1)
                .direc_iter(Direction::Right, |cd| cd.x >= bound_x)
                .collect()
        }
        _ => panic!(
            "[passages::connet_2rooms] invalid direction {:?}",
            direction
        ),
    }
}

/// node of room graph
struct Node {
    connections: RefCell<FixedBitSet>,
    candidates: HashMap<usize, Direction>,
    id: usize,
}

impl Node {
    fn new(xrooms: X, yrooms: Y, cd: (i32, i32), id: usize) -> Self {
        let candidates: HashMap<_, _> = Direction::iter_variants()
            .take(4)
            .filter_map(|d| {
                let next = cd.add(d.to_cd().into_tuple2());
                if next.any(|a| a < 0) || next.0 >= xrooms.0 || next.1 >= yrooms.0 {
                    return None;
                }
                Some(((next.0 + next.1 * xrooms.0) as usize, d))
            })
            .collect();
        let num_rooms = (xrooms.0 * yrooms.0) as usize;
        Node {
            connections: RefCell::new(FixedBitSet::with_capacity(num_rooms)),
            candidates,
            id,
        }
    }
}

#[cfg_attr(test, test)]
fn test_inclusive_edges() {
    use std::ops::Range;
    let range = RectRange::from_ranges(5..10, 6..9).unwrap();
    let edge_vec = |xfix, fix, range: Range<i32>| -> Vec<_> {
        if xfix {
            range.map(|y| Coord::new(fix, y)).collect()
        } else {
            range.map(|x| Coord::new(x, fix)).collect()
        }
    };
    assert_eq!(
        inclusive_edges(&range, Direction::Down),
        edge_vec(false, 8, 6..9)
    );
    assert_eq!(
        inclusive_edges(&range, Direction::Up),
        edge_vec(false, 6, 6..9)
    );
    assert_eq!(
        inclusive_edges(&range, Direction::Left),
        edge_vec(true, 5, 7..8)
    );
    assert_eq!(
        inclusive_edges(&range, Direction::Right),
        edge_vec(true, 9, 7..8)
    );
}
