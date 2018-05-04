use super::{maze, Config, Surface};
use dungeon::{Coord, Positioned, X, Y};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use fixedbitset::FixedBitSet;
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

/// A data structure representing a room in the dungeon
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
    pub fn draw<R>(&self, mut register: R) -> GameResult<()>
    where
        R: FnMut(Positioned<Surface>) -> GameResult<()>,
    {
        match self.kind {
            // TODO: door
            RoomKind::Normal { ref range } => range
                .iter()
                .try_for_each(|cd| {
                    let surface = if range.is_edge(cd) {
                        if cd.1 == range.upper_left().1 || cd.1 == range.lower_left().1 {
                            Surface::WallX
                        } else {
                            Surface::WallY
                        }
                    } else {
                        Surface::Floor
                    };
                    register(Positioned(cd.into(), surface))
                })
                .chain_err("[Room::draw]"),
            RoomKind::Maze {
                range: _,
                ref passages,
            } => passages
                .iter()
                .try_for_each(|&cd| register(Positioned(cd.into(), Surface::Passage)))
                .chain_err("[Room::draw]"),
            RoomKind::Empty { .. } => Ok(()),
        }
    }
}

/// generate rooms
pub(crate) fn gen_rooms(
    level: u32,
    config: &Config,
    width: X,
    height: Y,
    rng: &mut RngHandle,
) -> GameResult<Vec<Room>> {
    let (rn_x, rn_y) = (config.room_num_x, config.room_num_y);
    let room_num = (rn_x.0 * rn_y.0) as usize;
    // Be aware that it's **screen** size!
    let (width, height) = (width, height);
    let room_size = Coord::new(width / rn_x.0, height / rn_y.0);
    // set empty rooms
    let empty_rooms: FixedBitSet = {
        let empty_num = rng.range(0..config.max_empty_rooms) + 1;
        rng.select(0..room_num).take(empty_num as usize).collect()
    };
    RectRange::zero_start(rn_x.0, rn_y.0)
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, (x, y))| {
            let mut room_size = room_size;
            // adjust room positions so as not to hit the comment area
            let upper_left = if y == 0 {
                let res = room_size.scale(x, y).slide_y(1);
                room_size.y -= Y(1);
                res
            } else {
                room_size.scale(x, y)
            };
            if upper_left.y + room_size.y == height {
                room_size.y -= Y(1);
            }
            let is_empty = empty_rooms.contains(i);
            make_room(is_empty, room_size, upper_left, i, &config, level, rng)
        })
        .collect()
}

/// generata a room
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
        // maze
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
        RoomKind::Maze {
            range: range,
            passages: passages,
        }
    } else {
        // normal
        let size = {
            let (xmin, ymin) = config.min_room_size.into_tuple2();
            ((room_size.x.0, xmin), (room_size.y.0, ymin)).map(|(max, min)| rng.range(min..max))
        };
        let upper_left = (room_size.x.0, room_size.y.0)
            .sub(size)
            .map(|rest| rng.range(0..rest))
            .add(upper_left.into_tuple2());
        let room_range = RectRange::from_corners(upper_left, upper_left.add(size)).unwrap();
        RoomKind::Normal { range: room_range }
    };
    Ok(Room::new(kind, is_dark, id))
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use rect_iter::GetMut2D;
    use Tile;
    pub(crate) fn gen(level: u32) -> Vec<Room> {
        let config = Config::default();
        let (w, h) = (X(80), Y(24));
        let mut rng = RngHandle::new();
        gen_rooms(1, &config, w, h, &mut rng).unwrap()
    }
    pub(crate) fn draw_to_buffer(rooms: &[Room]) -> Vec<Vec<Surface>> {
        let mut buffer = vec![vec![Surface::None; 80]; 24];
        for room in rooms {
            room.draw(|Positioned(cd, s)| {
                *buffer.get_mut_p(cd) = s;
                Ok(())
            }).unwrap();
        }
        buffer
    }
    #[test]
    #[ignore]
    fn print_rooms() {
        let rooms = gen(10);
        let buffer = draw_to_buffer(&rooms);
        for v in buffer {
            for x in v {
                print!("{}", x.byte() as char);
            }
            println!("");
        }
    }
    #[test]
    fn gen_check() {}
}
