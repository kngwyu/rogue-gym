use super::{maze, Config, Surface};
use dungeon::{Coord, Positioned, X, Y};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use fenwick::FenwickSet;
use fixedbitset::FixedBitSet;
use rect_iter::{IntoTuple2, RectRange};
use rng::RngHandle;
use std::collections::HashSet;
use tuple_map::TupleMap2;
/// type of room
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomKind {
    /// normal room
    Normal { range: RectRange<i32> },
    /// maze room
    Maze {
        range: RectRange<i32>,
        passages: HashSet<Coord>,
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
    /// a range punctuated when generating rooms
    pub assigned_area: RectRange<i32>,
    /// if the player has visited the room or notify
    pub is_visited: bool,
    /// cells where we can set an object
    empty_cells: FenwickSet,
}

impl Room {
    fn new(kind: RoomKind, is_dark: bool, id: usize, assigned: RectRange<i32>) -> Self {
        let empty_cells = gen_empty_cells(&kind);
        Room {
            kind,
            is_dark,
            id,
            assigned_area: assigned,
            empty_cells,
            is_visited: false,
        }
    }
    /// takes a closure `register` and draw room by it
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
    /// return a range occupied by the room
    pub fn range(&self) -> Option<&RectRange<i32>> {
        match self.kind {
            RoomKind::Normal { ref range } | RoomKind::Maze { ref range, .. } => Some(range),
            _ => None,
        }
    }
    fn get_cell_id(&mut self, cd: Coord) -> Option<usize> {
        let range = self.range()?;
        range.index(cd)
    }
    /// modifiy the a cell's condition to 'not filled'
    pub fn empty_cell(&mut self, cd: Coord) -> bool {
        if let Some(id) = self.get_cell_id(cd) {
            self.empty_cells.remove(id)
        } else {
            false
        }
    }
    /// modifiy the a cell's condition to 'filled'
    pub fn fill_cell(&mut self, cd: Coord) -> bool {
        if let Some(id) = self.get_cell_id(cd) {
            self.empty_cells.insert(id)
        } else {
            false
        }
    }
    /// select a cell where we can set an object
    pub fn select_empty_cell(&self, rng: &mut RngHandle) -> Option<Coord> {
        self.range().and_then(|range| {
            let cell_n = self.empty_cells.select(rng)?;
            range.nth(cell_n).map(|t| Coord::from(t))
        })
    }
    pub fn is_normal(&self) -> bool {
        match self.kind {
            RoomKind::Normal { .. } => true,
            _ => false,
        }
    }
    pub fn is_maze(&self) -> bool {
        match self.kind {
            RoomKind::Maze { .. } => true,
            _ => false,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self.kind {
            RoomKind::Empty { .. } => true,
            _ => false,
        }
    }
}

fn gen_empty_cells(kind: &RoomKind) -> FenwickSet {
    match kind {
        RoomKind::Normal { range } => {
            let len = range.len();
            let mut set = FenwickSet::with_capacity(len);
            range.iter().enumerate().for_each(|(i, cd)| {
                if !range.is_edge(cd) {
                    set.insert(i);
                }
            });
            set
        }
        RoomKind::Maze { range, passages } => {
            let len = range.len();
            let mut set = FenwickSet::with_capacity(len);
            range.iter().enumerate().for_each(|(i, cd)| {
                let cd: Coord = cd.into();
                if !passages.contains(&cd) {
                    set.insert(i);
                }
            });
            set
        }
        RoomKind::Empty { .. } => FenwickSet::with_capacity(1),
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
                room_size.y -= Y(1);
                room_size.scale(x, y).slide_y(1)
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
    let assigned_range = RectRange::from_corners(upper_left, upper_left + room_size).unwrap();
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
            assigned_range,
        ));
    }
    let is_dark = rng.range(0..config.dark_level) + 1 < level;
    let kind = if is_dark && rng.does_happen(config.maze_rate_inv) {
        // maze
        let range =
            RectRange::from_corners(upper_left, upper_left + room_size - Coord::new(1, 1)).unwrap();
        let mut passages = HashSet::new();
        maze::dig_maze(range.clone(), rng, |cd| {
            if range.contains(cd) {
                passages.insert(cd);
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
        // Take care that doors is empty at this phase
        RoomKind::Normal { range: room_range }
    };
    Ok(Room::new(kind, is_dark, id, assigned_range))
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use dungeon::Direction;
    use rect_iter::GetMut2D;
    use tile::Drawable;
    pub(crate) fn gen(level: u32) -> Vec<Room> {
        let config = Config::default();
        let (w, h) = (X(80), Y(24));
        let mut rng = RngHandle::new();
        gen_rooms(level, &config, w, h, &mut rng).unwrap()
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
                print!("{}", x.tile())
            }
            println!("");
        }
    }
    #[test]
    fn check() {
        let (xrooms, yrooms) = (3, 3);
        for i in 0..1000 {
            let rooms = gen(i % 20);
            for (x, y) in RectRange::zero_start(xrooms, yrooms).unwrap() {
                let room1 = &rooms[x + xrooms * y];
                Direction::iter_variants().take(4).for_each(|d| {
                    let (nx, ny) = d.to_cd().into_tuple2().add((x as i32, y as i32));
                    if nx < 0 || ny < 0 || nx >= xrooms as i32 || ny >= yrooms as i32 {
                        return;
                    }
                    let (nx, ny) = (nx, ny).map(|i| i as usize);
                    let room2 = &rooms[nx + xrooms * ny];
                    if let Some(r1) = room1.range() {
                        assert!(r1.area() >= 9);
                        if let Some(r2) = room2.range() {
                            assert!(r2.area() >= 9);
                            let diff = match d {
                                Direction::Up => r1.upper_left().1 - r2.lower_right().1,
                                Direction::Right => r2.upper_left().0 - r1.lower_right().0,
                                Direction::Left => r1.upper_left().0 - r2.lower_right().0,
                                Direction::Down => r2.upper_left().1 - r1.lower_right().1,
                                _ => unreachable!(),
                            };
                            assert!(diff >= 1, "{:?}", diff);
                        }
                    }
                });
            }
        }
    }
}
