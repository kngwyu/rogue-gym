pub mod floor;
pub mod maze;
pub mod passages;
pub mod rooms;

use self::floor::Floor;
pub use self::rooms::{Room, RoomKind};
use super::{Coord, Direction, DungeonPath, Positioned, X, Y};
use error::*;
use item::{ItemHandler, ItemToken};
use ndarray::Array2;
use rect_iter::{Get2D, GetMut2D, RectRange};
use rng::RngHandle;
use tile::{Drawable, Tile};
use {GameInfo, GameMsg, GlobalConfig};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    /// room number in X-axis direction
    #[serde(default = "default_room_num_x")]
    pub room_num_x: X,
    /// room number in X-axis direction
    #[serde(default = "default_room_num_y")]
    pub room_num_y: Y,
    /// minimum size of a room
    #[serde(default = "default_min_room_size")]
    pub min_room_size: Coord,
    /// enables trap or not
    #[serde(default = "default_trap")]
    pub enable_trap: bool,
    /// maximum number of empty rooms
    #[serde(default = "default_max_empty_rooms")]
    pub max_empty_rooms: u32,
    /// the level where the Amulet of Yendor is
    #[serde(default = "default_amulet_level")]
    pub amulet_level: u32,
    /// a room changes to maze with a probability of 1 / maze_rate_inv
    #[serde(default = "default_maze_rate")]
    pub maze_rate_inv: u32,
    /// if the rooms is dark or not is judged by rand[0..dark_level) < level - 1
    #[serde(default = "default_dark_level")]
    pub dark_level: u32,
    /// a passage is hidden with a probability of 1 / hidden_rate_inv
    #[serde(default = "default_hidden_passage_rate")]
    pub hidden_passage_rate_inv: u32,
    /// a door is locked with a probability of 1 / hidden_rate_inv
    #[serde(default = "default_locked_door_rate_inv")]
    pub locked_door_rate_inv: u32,
    /// try number of additional passages
    #[serde(default = "default_max_extra_edges")]
    pub max_extra_edges: u32,
    #[serde(default = "default_door_unlock_rate_inv")]
    pub door_unlock_rate_inv: u32,
    #[serde(default = "default_passage_unlock_rate_inv")]
    pub passage_unlock_rate_inv: u32,
}

const fn default_room_num_x() -> X {
    X(3)
}

const fn default_room_num_y() -> Y {
    Y(3)
}

#[inline]
fn default_min_room_size() -> Coord {
    Coord::new(4, 4)
}

const fn default_trap() -> bool {
    true
}

const fn default_max_empty_rooms() -> u32 {
    3
}

const fn default_amulet_level() -> u32 {
    25
}

const fn default_maze_rate() -> u32 {
    15
}

const fn default_dark_level() -> u32 {
    10
}

const fn default_hidden_passage_rate() -> u32 {
    40
}

const fn default_locked_door_rate_inv() -> u32 {
    5
}

const fn default_max_extra_edges() -> u32 {
    5
}

const fn default_door_unlock_rate_inv() -> u32 {
    5
}

const fn default_passage_unlock_rate_inv() -> u32 {
    3
}

impl Default for Config {
    fn default() -> Config {
        Config {
            room_num_x: default_room_num_x(),
            room_num_y: default_room_num_y(),
            min_room_size: default_min_room_size(),
            enable_trap: default_trap(),
            max_empty_rooms: default_max_empty_rooms(),
            amulet_level: default_amulet_level(),
            maze_rate_inv: default_maze_rate(),
            dark_level: default_dark_level(),
            hidden_passage_rate_inv: default_hidden_passage_rate(),
            locked_door_rate_inv: default_locked_door_rate_inv(),
            max_extra_edges: default_max_extra_edges(),
            door_unlock_rate_inv: default_door_unlock_rate_inv(),
            passage_unlock_rate_inv: default_passage_unlock_rate_inv(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Surface {
    Passage,
    Floor,
    WallX,
    WallY,
    Stair,
    Door,
    Trap,
    None,
}

impl Drawable for Surface {
    fn tile(&self) -> Tile {
        match *self {
            Surface::Passage => b'#',
            Surface::Floor => b'.',
            Surface::WallX => b'-',
            Surface::WallY => b'|',
            Surface::Stair => b'%',
            Surface::Door => b'+',
            Surface::Trap => b'^',
            Surface::None => b' ',
        }
        .into()
    }
}

impl Default for Surface {
    fn default() -> Surface {
        Surface::None
    }
}

impl Surface {
    fn can_walk(&self) -> bool {
        match *self {
            Surface::WallX | Surface::WallY | Surface::None => false,
            _ => true,
        }
    }
    fn wall(is_xdir: bool) -> Self {
        if is_xdir {
            Surface::WallX
        } else {
            Surface::WallY
        }
    }
}

/// representation of rogue dungeon
#[derive(Clone)]
pub struct Dungeon {
    /// current level
    pub level: u32,
    /// amulet level or more deeper level the player visited
    pub max_level: u32,
    /// current floor
    pub current_floor: Floor,
    /// dungeon specific configuration(constant)
    pub config: Config,
    /// global configuration(constant)
    pub config_global: GlobalConfig,
    /// past floors
    pub past_floors: Vec<Floor>,
    /// random number generator
    pub rng: RngHandle,
}

impl Dungeon {
    /// make new dungeon
    pub fn new(
        config: Config,
        config_global: &GlobalConfig,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
        seed: u128,
    ) -> GameResult<Self> {
        let rng = RngHandle::from_seed(seed);
        let mut dungeon = Dungeon {
            level: 0,
            max_level: config.amulet_level,
            current_floor: Floor::default(),
            config,
            config_global: config_global.clone(),
            past_floors: vec![],
            rng,
        };
        dungeon
            .new_level_(game_info, item_handle, true)
            .chain_err(|| "rogue::Dungeon::new")?;
        Ok(dungeon)
    }

    /// returns the range to draw
    crate fn draw_ranges<'a>(&'a self) -> impl 'a + Iterator<Item = DungeonPath> {
        let level = self.level;
        let xmax = self.config_global.width.0;
        let ymax = self.config_global.height.0 - 1;
        RectRange::from_ranges(0..xmax, 1..ymax)
            .unwrap()
            .into_iter()
            .filter(move |&cd| self.current_floor.field.get_p(cd).is_obj_visible())
            .map(move |cd| [level as i32, cd.0, cd.1].into())
    }

    /// setup next floor
    pub fn new_level(
        &mut self,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
    ) -> GameResult<()> {
        self.new_level_(game_info, item_handle, false)
    }

    fn new_level_(
        &mut self,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
        is_initial: bool,
    ) -> GameResult<()> {
        const ERR_STR: &str = "in rogue::Dungeon::new_level";
        let level = {
            self.level += 1;
            self.level
        };
        if level > self.max_level {
            self.max_level = level;
        }
        let (width, height) = (self.config_global.width, self.config_global.height);
        let mut floor = Floor::gen_floor(level, &self.config, width, height, &mut self.rng)
            .chain_err(|| ERR_STR)?;
        debug!("[Dungeon::new_level] field: {}", floor.field);
        // setup gold
        let set_gold = !game_info.is_cleared || level >= self.max_level;
        debug!("[Dungeon::new_level] set_gold: {}", set_gold);
        floor.setup_items(level, item_handle, set_gold, &mut self.rng);
        // place traps (STUB)
        // place stair
        floor.setup_stair(&mut self.rng).chain_err(|| ERR_STR)?;
        if !self.config_global.hide_dungeon {
            let xmax = self.config_global.width.0;
            let ymax = self.config_global.height.0 - 1;
            RectRange::from_ranges(0..xmax, 1..ymax)
                .unwrap()
                .into_iter()
                .for_each(|cd| {
                    let cell = floor.field.get_mut_p(cd);
                    cell.visible(true);
                });
        }
        ::std::mem::swap(&mut self.current_floor, &mut floor);
        if !is_initial {
            self.past_floors.push(floor);
        }
        Ok(())
    }

    /// takes addrees and judge if thers's a stair at that address
    crate fn is_downstair(&self, address: Address) -> bool {
        if address.level != self.level {
            return false;
        }
        if let Ok(cell) = self.current_floor.field.try_get_p(address.cd) {
            cell.surface == Surface::Stair
        } else {
            false
        }
    }

    /// judge if the player can move from the address in the direction
    crate fn can_move_player(&self, address: Address, direction: Direction) -> bool {
        if address.level != self.level {
            return false;
        }
        self.current_floor.can_move_player(address.cd, direction)
    }
    /// move the player from the address in the direction
    crate fn move_player(
        &mut self,
        address: Address,
        direction: Direction,
    ) -> GameResult<DungeonPath> {
        const ERR_STR: &str = "[rogue::Dungeon::move_player]";
        if address.level != self.level {
            return Err(ErrorId::MaybeBug.into_with(|| ERR_STR));
        }
        self.current_floor
            .player_out(address.cd)
            .chain_err(|| ERR_STR)?;
        let cd = address.cd + direction.to_cd();
        let address = Address {
            level: self.level,
            cd,
        };
        self.current_floor
            .player_in(cd, false)
            .chain_err(|| ERR_STR)?;
        Ok(address.into())
    }
    crate fn search<'a>(
        &'a mut self,
        address: Address,
    ) -> GameResult<impl 'a + Iterator<Item = GameMsg>> {
        if address.level != self.level {
            return Err(ErrorId::MaybeBug.into_with(|| "[rogue::Dungeon::search]"));
        }
        Ok(self
            .current_floor
            .search(address.cd, &mut self.rng, &self.config))
    }
    /// select an empty cell randomly
    crate fn select_cell(&mut self, is_character: bool) -> Option<DungeonPath> {
        self.current_floor
            .select_cell(&mut self.rng, is_character)
            .map(|cd| [self.level as i32, cd.x.0, cd.y.0].into())
    }
    crate fn remove_object(&mut self, address: Address, is_character: bool) -> bool {
        self.current_floor.remove_obj(address.cd, is_character)
    }
    /// draw dungeon to screen by callback
    crate fn draw<F>(&self, drawer: &mut F) -> GameResult<()>
    where
        F: FnMut(Positioned<Tile>) -> GameResult<()>,
    {
        const ERR_STR: &str = "in rogue::Dungeon::move_player";
        let range = self
            .current_floor
            .field
            .size_ytrimed()
            .ok_or_else(|| ErrorId::MaybeBug.into_with(|| ERR_STR))?;
        range.into_iter().try_for_each(|cd| {
            let cd = Coord::from(cd);
            let cell = self.current_floor.field.try_get_p(cd)?;
            drawer(Positioned(cd, cell.tile()))
        })
    }
    crate fn get_item(&self, addr: Address) -> Option<&ItemToken> {
        if addr.level != self.level {
            return None;
        }
        self.current_floor.items.get(&addr.cd)
    }
    crate fn remove_item(&mut self, addr: Address) -> Option<ItemToken> {
        if addr.level != self.level {
            return None;
        }
        if !self.current_floor.remove_obj(addr.cd, false) {
            return None;
        }
        self.current_floor.items.remove(&addr.cd)
    }
    crate fn tile(&mut self, addr: Address) -> Option<Tile> {
        self.current_floor
            .field
            .try_get_mut_p(addr.cd)
            .ok()
            .map(|s| s.tile())
    }
    crate fn gen_history_map(&self, level: u32) -> Option<Array2<bool>> {
        if level == self.level {
            Some(self.current_floor.history_map())
        } else if let Some(floor) = self.past_floors.get(level as usize - 1) {
            Some(floor.history_map())
        } else {
            None
        }
    }
}

/// Address in the dungeon.
/// It's quite simple in rogue.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
crate struct Address {
    /// level
    crate level: u32,
    /// coordinate
    crate cd: Coord,
}

impl Address {
    crate fn from_path(p: &DungeonPath) -> Address {
        Address {
            level: p[0] as u32,
            cd: Coord::new(p[1], p[2]),
        }
    }
}
