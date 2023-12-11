use super::{maze, Config, Surface};
use crate::dungeon::{Coord, Positioned, X, Y};
use crate::{error::*, fenwick::FenwickSet, rng::RngHandle};
use anyhow::{bail, Context};
use fixedbitset::FixedBitSet;
use log::warn;
use rect_iter::{IntoTuple2, RectRange};
use tuple_map::TupleMap2;

/// type of room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RoomKind {
    /// normal room
    Normal { range: RectRange<i32> },
    /// maze room
    Maze(Box<maze::Maze>),
    /// passage only(gone room)
    Empty { up_left: Coord },
}

/// A data structure representing a room in the dungeon
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    /// room kindQ
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
    /// if this room has gold or not
    pub has_gold: bool,
    /// cells which has no object
    empty_cells: FenwickSet,
    /// cells which has no enemy
    nocharacter_cells: FenwickSet,
}

impl Room {
    fn new(kind: RoomKind, is_dark: bool, id: usize, assigned: RectRange<i32>) -> Self {
        let empty_cells = gen_empty_cells(&kind);
        Room {
            kind,
            is_dark,
            id,
            assigned_area: assigned,
            nocharacter_cells: empty_cells.clone(),
            empty_cells,
            has_gold: false,
            is_visited: false,
        }
    }
    /// takes a closure `register` and draw room by it
    pub fn draw<R>(&self, mut register: R) -> GameResult<()>
    where
        R: FnMut(Positioned<Surface>) -> GameResult<()>,
    {
        match self.kind {
            RoomKind::Normal { ref range } => range
                .iter()
                .try_for_each(|cd| {
                    let surface = if range.is_horiz_edge(cd) {
                        Surface::WallX
                    } else if range.is_vert_edge(cd) {
                        Surface::WallY
                    } else {
                        Surface::Floor
                    };
                    register(Positioned(cd.into(), surface))
                })
                .context("Room::draw"),
            RoomKind::Maze(ref maze) => maze
                .passages()
                .try_for_each(|cd| register(Positioned(cd, Surface::Passage)))
                .context("Room::draw"),
            RoomKind::Empty { .. } => Ok(()),
        }
    }
    /// Returns the 'room' range
    pub fn range(&self) -> Option<&RectRange<i32>> {
        match self.kind {
            RoomKind::Normal { ref range } => Some(range),
            RoomKind::Maze(ref maze) => Some(&maze.range),
            _ => None,
        }
    }
    fn get_cell_id(&mut self, cd: Coord) -> Option<usize> {
        let range = self.range()?;
        range.index(cd)
    }
    /// modifiy the a cell's condition to 'filled'
    pub fn fill_cell(&mut self, cd: Coord, is_character: bool) -> bool {
        if let Some(id) = self.get_cell_id(cd) {
            if is_character {
                self.nocharacter_cells.remove(id);
            }
            self.empty_cells.remove(id)
        } else {
            false
        }
    }
    /// modifiy the a cell's condition to 'unfilled'
    pub fn unfill_cell(&mut self, cd: Coord, is_character: bool) -> bool {
        if let Some(id) = self.get_cell_id(cd) {
            if is_character {
                self.nocharacter_cells.insert(id);
            }
            self.empty_cells.insert(id)
        } else {
            false
        }
    }
    pub fn is_normal(&self) -> bool {
        match self.kind {
            RoomKind::Normal { .. } => true,
            _ => false,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self.kind {
            RoomKind::Empty { .. } => true,
            _ => false,
        }
    }
    pub fn contains(&self, cd: Coord) -> bool {
        self.assigned_area.contains(cd)
    }
    fn select_cell_impl(&self, set: &FenwickSet, rng: &mut RngHandle) -> Option<Coord> {
        self.range().and_then(|range| {
            let cell_n = set.select(rng)?;
            range.nth(cell_n).map(Coord::from)
        })
    }
    pub fn select_cell(&self, rng: &mut RngHandle, is_character: bool) -> Option<Coord> {
        if is_character {
            self.select_cell_impl(&self.nocharacter_cells, rng)
        } else {
            self.select_cell_impl(&self.empty_cells, rng)
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
        RoomKind::Maze(ref maze) => maze.passages.clone(),
        RoomKind::Empty { .. } => FenwickSet::with_capacity(1),
    }
}

/// generate rooms
pub(super) fn gen_rooms(
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
        let empty_num = match rng.range(0..=config.max_empty_rooms) {
            n if n >= room_num as u32 => {
                warn!(
                    "Specified max_empty_rooms is {}, but room num is {}",
                    n, room_num
                );
                room_num as u32 - 1
            }
            n => n,
        };
        rng.select(0..room_num).take(empty_num as usize).collect()
    };
    RectRange::zero_start(rn_x.0, rn_y.0)
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, (x, y))| {
            let mut room_size = room_size;
            // adjust room positions so as not to hit the comment area
            let lower_left = if y == 0 {
                room_size.y -= Y(1);
                room_size.scale(x, y).slide_y(1)
            } else {
                room_size.scale(x, y)
            };
            if lower_left.y + room_size.y == height {
                room_size.y -= Y(1);
            }
            let is_empty = empty_rooms.contains(i);
            make_room(is_empty, room_size, lower_left, i, &config, level, rng)
        })
        .collect()
}

/// generata a room
pub(super) fn make_room(
    is_empty: bool,
    room_size: Coord,
    lower_left: Coord,
    id: usize,
    config: &Config,
    level: u32,
    rng: &mut RngHandle,
) -> GameResult<Room> {
    let assigned_range = RectRange::from_corners(lower_left, lower_left + room_size).unwrap();
    if is_empty {
        let (x, y) = (room_size.x.0, room_size.y.0)
            .map(|size| rng.range(1..size - 1))
            .add(lower_left.into_tuple2());
        return Ok(Room::new(
            RoomKind::Empty {
                up_left: Coord::new(x, y),
            },
            true,
            id,
            assigned_range,
        ));
    }
    let is_dark = rng.range(0..config.dark_level) < level;
    let kind = if is_dark && rng.does_happen(config.maze_rate_inv) {
        // maze
        let range =
            RectRange::from_corners(lower_left, lower_left + room_size - Coord::new(1, 1)).unwrap();
        let len = range.len();
        let mut passages = FenwickSet::with_capacity(len);
        maze::dig_maze(range.clone(), rng, |cd| {
            if let Some(id) = range.index(cd) {
                passages.insert(id);
                Ok(())
            } else {
                bail!(ErrorKind::MaybeBug("dig_maze produced invalid Coordinate!"));
            }
        })?;
        let maze = maze::Maze { range, passages };
        RoomKind::Maze(Box::new(maze))
    } else {
        // normal
        let size = {
            let (xmin, ymin) = config.min_room_size.into_tuple2();
            ((room_size.x.0, xmin), (room_size.y.0, ymin)).map(|(max, min)| rng.range(min..max))
        };
        let lower_left = (room_size.x.0, room_size.y.0)
            .sub(size)
            .map(|rest| rng.range(0..rest))
            .add(lower_left.into_tuple2());
        let room_range = RectRange::from_corners(lower_left, lower_left.add(size)).unwrap();
        // Take care that doors is empty at this phase
        RoomKind::Normal { range: room_range }
    };
    Ok(Room::new(kind, is_dark, id, assigned_range))
}

#[cfg(test)]
pub(super) mod test {
    use super::*;
    use crate::dungeon::Direction;
    use crate::tile::Drawable;
    use rect_iter::GetMut2D;
    pub fn gen(level: u32) -> Vec<Room> {
        let mut config = Config::default();
        config.maze_rate_inv = 5;
        let (w, h) = (X(80), Y(24));
        let mut rng = RngHandle::new();
        gen_rooms(level, &config, w, h, &mut rng).unwrap()
    }
    pub fn draw_to_buffer(rooms: &[Room]) -> Vec<Vec<Surface>> {
        let mut buffer = vec![vec![Surface::None; 80]; 24];
        for room in rooms {
            room.draw(|Positioned(cd, s)| {
                *buffer.get_mut_p(cd) = s;
                Ok(())
            })
            .unwrap();
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
            println!();
        }
    }
    #[test]
    fn pos_check() {
        let (xrooms, yrooms) = (3, 3);
        use enum_iterator::IntoEnumIterator;
        for i in 0..100 {
            let rooms = gen(i % 20);
            for (x, y) in RectRange::zero_start(xrooms, yrooms).unwrap() {
                let room1 = &rooms[x + xrooms * y];
                Direction::into_enum_iter().take(4).for_each(|d| {
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
                                Direction::Up => r1.lower_left().1 - r2.upper_right().1,
                                Direction::Right => r2.lower_left().0 - r1.upper_right().0,
                                Direction::Left => r1.lower_left().0 - r2.upper_right().0,
                                Direction::Down => r2.lower_left().1 - r1.upper_right().1,
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
