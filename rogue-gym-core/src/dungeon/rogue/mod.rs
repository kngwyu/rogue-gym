pub mod floor;
pub mod maze;
pub mod passages;
pub mod rooms;

use self::floor::Floor;
pub use self::rooms::{Room, RoomKind};
use super::{Coord, Direction, DungeonPath, Positioned, X, Y};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use error_chain_mini::ChainedError;
use item::ItemHandler;
use rect_iter::{Get2D, RectRange};
use rng::RngHandle;
use tile::{Drawable, Tile};
use {GameInfo, GlobalConfig};

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
    /// if the rooms is dark or not is judged by rand[0..dark_levl) < level - 1
    #[serde(default = "default_dark_level")]
    pub dark_level: u32,
    /// a passage is hidden with a probability of 1 / hidden_rate_inv
    #[serde(default = "default_hidden_passage_rate")]
    pub hidden_passage_rate_inv: u32,
    /// a door is locked with a probability of 1 / hidden_rate_inv
    #[serde(default = "default_locked_door_rate")]
    pub locked_door_rate_inv: u32,
    /// try number of additional passages
    #[serde(default = "default_max_extra_edges")]
    pub max_extra_edges: u32,
}

#[inline]
fn default_room_num_x() -> X {
    X(3)
}
#[inline]
fn default_room_num_y() -> Y {
    Y(3)
}
#[inline]
fn default_min_room_size() -> Coord {
    Coord::new(4, 4)
}
#[inline]
fn default_trap() -> bool {
    true
}
#[inline]
fn default_max_empty_rooms() -> u32 {
    4
}
#[inline]
fn default_amulet_level() -> u32 {
    25
}
#[inline]
fn default_maze_rate() -> u32 {
    15
}
#[inline]
fn default_dark_level() -> u32 {
    10
}
#[inline]
fn default_hidden_passage_rate() -> u32 {
    40
}
#[inline]
fn default_locked_door_rate() -> u32 {
    5
}
#[inline]
fn default_max_extra_edges() -> u32 {
    5
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
            locked_door_rate_inv: default_locked_door_rate(),
            max_extra_edges: default_max_extra_edges(),
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
        }.into()
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
}

/// representation of rogue dungeon
#[derive(Clone, Serialize, Deserialize)]
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
        seed: u64,
    ) -> GameResult<Self> {
        let rng = RngHandle::from_seed(seed);
        let mut dungeon = Dungeon {
            level: 1,
            max_level: config.amulet_level,
            current_floor: Floor::default(),
            config,
            config_global: config_global.clone(),
            rng,
        };
        dungeon
            .new_level(game_info, item_handle)
            .chain_err("rogue::Dungeon::new")?;
        Ok(dungeon)
    }
    pub fn draw_ranges(&self) -> impl Iterator<Item = DungeonPath> {
        let level = self.level;
        let xmax = self.config_global.width.0;
        let ymax = self.config_global.height.0 - 1;
        RectRange::from_ranges(0..xmax, 1..ymax)
            .unwrap()
            .into_iter()
            .map(move |cd| vec![level as i32, cd.0, cd.1].into())
    }
    /// setup next floor
    pub fn new_level(
        &mut self,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
    ) -> GameResult<()> {
        let level = {
            self.level += 1;
            self.level
        };
        if level > self.max_level {
            self.max_level = level;
        }
        let (width, height) = (self.config_global.width, self.config_global.height);
        let floor = Floor::gen_floor(level, &self.config, width, height, &mut self.rng)
            .chain_err("Dungeon::new_floor")?;
        // setup gold
        let set_gold = game_info.is_cleared || level >= self.max_level;
        self.current_floor
            .setup_items(level, item_handle, set_gold, &mut self.rng)
            .chain_err("Dungeon::new_floor")?;
        self.current_floor = floor;
        Ok(())
    }
    /// takes addrees and judge if thers's a stair at that address
    pub(crate) fn is_downstair(&self, address: Address) -> bool {
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
    pub(crate) fn can_move_player(&self, address: Address, direction: Direction) -> bool {
        if address.level != self.level {
            return false;
        }
        self.current_floor.can_move_player(address.cd, direction)
    }
    /// move the player from the address in the direction
    pub(crate) fn move_player(
        &mut self,
        address: Address,
        direction: Direction,
    ) -> GameResult<DungeonPath> {
        const ERR_STR: &str = "[rogue::Dungeon::move_player]";
        if address.level != self.level {
            return Err(ErrorId::MaybeBug.into_with(ERR_STR));
        }
        self.current_floor
            .player_out(address.cd)
            .chain_err(ERR_STR)?;
        let cd = address.cd + direction.to_cd();
        let address = Address {
            level: self.level,
            cd,
        };
        self.current_floor.player_in(cd).chain_err(ERR_STR)?;
        Ok(address.into())
    }
    /// select an empty cell randomly
    pub(crate) fn select_cell(&mut self, is_character: bool) -> Option<DungeonPath> {
        self.current_floor
            .select_cell(&mut self.rng, is_character)
            .map(|cd| vec![self.level as i32, cd.x.0, cd.y.0].into())
    }
    /// draw dungeon to screen by callback
    pub(crate) fn draw<F, E>(&self, drawer: &mut F) -> Result<(), ChainedError<E>>
    where
        F: FnMut(Positioned<Tile>) -> Result<(), ChainedError<E>>,
        E: From<ErrorId> + ErrorKind,
    {
        const ERR_STR: &str = "in rogue::Dungeon::move_player";
        let range = self.current_floor
            .field
            .size()
            .ok_or_else(|| E::from(ErrorId::MaybeBug).into_with(ERR_STR))?;
        range.into_iter().try_for_each(|cd| {
            let cell = self.current_floor
                .field
                .try_get_p(cd)
                .into_chained(ERR_STR)
                .convert()?;
            let cd = Coord::from(cd);
            let tile = cell.tile();
            drawer(Positioned(cd, tile))
        })
    }
}

/// Address in the dungeon.
/// It's quite simple in rogue.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct Address {
    /// level
    pub(crate) level: u32,
    /// coordinate
    pub(crate) cd: Coord,
}

impl From<DungeonPath> for Address {
    fn from(d: DungeonPath) -> Address {
        assert!(d.0.len() == 3, "Address::from invalid value {:?}", d);
        Address {
            level: d.0[0] as u32,
            cd: Coord::new(d.0[1], d.0[2]),
        }
    }
}
