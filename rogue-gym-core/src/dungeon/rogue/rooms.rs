use super::{maze, Config};
use dungeon::Coord;
use error::{ErrorId, ErrorKind, GameResult};
use rect_iter::{IntoTuple2, RectRange};
use rng::RngHandle;
use tuple_map::TupleMap2;
/// type of room
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomKind {
    /// normal room
    Normal { range: RectRange<i32> },
    /// maze room
    Maze {
        range: RectRange<i32>,
        passages: Vec<Coord>,
    },
    /// passage only(gone room)
    Empty { up_left: Coord },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    /// room kind
    pub kind: RoomKind,
    /// if the room is dark or not
    pub is_dark: bool,
    /// id for room
    /// it's unique in same floor
    pub id: usize,
}

impl Room {
    fn new(kind: RoomKind, is_dark: bool, id: usize) -> Self {
        Room { kind, is_dark, id }
    }
    fn edges(&self) -> Vec<Coord> {
        vec![]
    }
}

pub(crate) fn make_room(
    is_empty: bool,
    room_size: Coord,
    upper_left: Coord,
    id: usize,
    config: &Config,
    level: u32,
    rng: &mut RngHandle,
) -> GameResult<Room> {
    if is_empty {
        let (x, y) = (room_size.x.0, room_size.y.0)
            .map(|size| rng.range(1..size - 1))
            .add(upper_left.into_tuple2());
        return Ok(Room::new(
            RoomKind::Empty {
                up_left: Coord::new(x, y),
            },
            true,
            id,
        ));
    }
    let is_dark = rng.range(0..config.dark_level) + 1 < level;
    let kind = if is_dark && rng.does_happen(config.maze_rate_inv) {
        let range = RectRange::from_corners(upper_left, upper_left + room_size).unwrap();
        let mut passages = Vec::new();
        maze::dig_maze(range.clone(), rng, |cd| {
            if range.contains(cd) {
                passages.push(cd);
                Ok(())
            } else {
                Err(ErrorId::LogicError.into_with("dig_maze produced invalid Coordinate!"))
            }
        })?;
        // maze
        RoomKind::Maze {
            range: range,
            // STUB
            passages: passages,
        }
    } else {
        // normal
        let (xsize, ysize) = {
            let (xmin, ymin) = config.min_room_size.into_tuple2();
            ((room_size.x.0, xmin), (room_size.y.0, ymin)).map(|(max, min)| rng.range(min..max))
        };
        let room_range =
            RectRange::from_corners(upper_left, upper_left + Coord::new(xsize, ysize)).unwrap();
        RoomKind::Normal { range: room_range }
    };
    Ok(Room::new(kind, is_dark, id))
}

// reserved code of item generation

// floor_range = room_range - wall_range
// let floor_range = room_range.clone().slide_start((1, 1)).slide_end((1, 1));
// let floor_num = floor_range.len() as usize;
// let cleared = self.game_info.borrow().is_cleared;
// if !cleared || level >= self.config.amulet_level {
//     self.item_handle.borrow_mut().setup_for_room(
//         floor_range.clone(),
//         level,
//         |item_rc| {
//             let selected = self.rng.borrow_mut().range(0..floor_num);
//             let coord = floor_range
//                 .nth(selected)
//                 .expect("[Dungeon::gen_floor] Invalid floor_num")
//                 .into();
//             item_map.insert(coord, item_rc);
//         },
//     );
// }
