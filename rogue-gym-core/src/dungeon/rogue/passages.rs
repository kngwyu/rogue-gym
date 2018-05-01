use super::Room;
use dungeon::{Direction, X, Y};
use fixedbitset::FixedBitSet;
use rect_iter::{IntoTuple2, RectRange};
use rng::RngHandle;
use std::cell::RefCell;
use tuple_map::TupleMap2;
/// make passages between rooms
pub(crate) fn dig_passges(rooms: &[Room], xrooms: X, yrooms: Y, rng: &mut RngHandle) {
    let range = RectRange::zero_start(xrooms.0, yrooms.0).unwrap();
    let mut graph: Vec<_> = range
        .into_iter()
        .enumerate()
        .map(|(i, t)| Node::new(xrooms, yrooms, t, i))
        .collect();
    let num_rooms = rooms.len();
    let mut selected = FixedBitSet::with_capacity(num_rooms);
    let current = rng.range(0..num_rooms);
    selected.insert(current);
    loop {
        let nxt_room = (0..num_rooms)
            .filter(|&i| graph[current].can_connect(i) && !selected.contains(i))
            .enumerate()
            .filter(|(i, _)| rng.does_happen(*i as u32 + 1))
            .last()
            .map(|t| t.1);
        if let Some(i) = nxt_room {

        } else {

        }
    }
}

/// node of room graph
struct Node {
    connections: RefCell<FixedBitSet>,
    candidates: FixedBitSet,
    id: usize,
}

impl Node {
    fn new(xrooms: X, yrooms: Y, cd: (i32, i32), id: usize) -> Self {
        let candidates: FixedBitSet = Direction::iter_variants()
            .take(4)
            .filter_map(|d| {
                let next = cd.add(d.to_cd().into_tuple2());
                if next.any(|a| a < 0) || next.0 >= xrooms.0 || next.1 >= yrooms.0 {
                    return None;
                }
                Some((next.0 + next.1 * xrooms.0) as usize)
            })
            .collect();
        let num_rooms = (xrooms.0 * yrooms.0) as usize;
        Node {
            connections: RefCell::new(FixedBitSet::with_capacity(num_rooms)),
            candidates,
            id,
        }
    }
    fn can_connect(&self, node_id: usize) -> bool {
        self.candidates.contains(node_id)
    }
}
