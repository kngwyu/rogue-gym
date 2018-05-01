use super::Room;
use dungeon::{Direction, X, Y};
use rect_iter::{IntoTuple2, RectRange};
use rng::RngHandle;
use std::cell::RefCell;
use tuple_map::TupleMap2;
/// make passages between rooms
pub(crate) fn dig_passges(rooms: &[Room], xrooms: X, yrooms: Y, rng: &mut RngHandle) {
    let range = RectRange::zero_start(xrooms.0, yrooms.0).unwrap();
    let graph: Vec<_> = range
        .into_iter()
        .map(|t| Node::new(xrooms, yrooms, t))
        .collect();
    let num_rooms = rooms.len();
    let selecter = rng.select(0..num_rooms).reserve();
}

/// node of room graph
struct Node {
    connected: RefCell<Vec<usize>>,
    can_connect: Vec<usize>,
}

impl Node {
    fn new(xrooms: X, yrooms: Y, cd: (i32, i32)) -> Self {
        let can_connect: Vec<_> = Direction::iter_variants()
            .take(4)
            .filter_map(|d| {
                let next = cd.add(d.to_cd().into_tuple2());
                if next.any(|a| a < 0) || next.0 >= xrooms.0 || next.1 >= yrooms.0 {
                    return None;
                }
                Some((next.0 + next.1 * xrooms.0) as usize)
            })
            .collect();
        Node {
            connected: RefCell::new(Vec::new()),
            can_connect,
        }
    }
}
