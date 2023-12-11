pub mod floor;
pub mod maze;
pub mod passages;
pub mod rooms;

use self::floor::Floor;
pub use self::rooms::{Room, RoomKind};
use crate::character::{player::Status as PlayerStatus, EnemyHandler};
use crate::dungeon::{
    Coord, Direction, Dungeon as DungeonTrait, DungeonPath, MoveResult, Positioned, X, Y,
};
use crate::item::{ItemHandler, ItemToken};
use crate::tile::{Drawable, Tile};
use crate::{error::*, rng::RngHandle, GameInfo, GameMsg, GlobalConfig};
use anyhow::{bail, Context};
use enum_iterator::IntoEnumIterator;
use ndarray::Array2;
use rect_iter::{Get2D, GetMut2D, RectRange};
use std::collections::VecDeque;
use tuple_map::TupleMap2;

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

    const NONE: Tile = Tile(b' ');

    fn color(&self) -> crate::tile::Color {
        crate::tile::Color(0)
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
    dist_cache: DistCache,
}

impl DungeonTrait for Dungeon {
    fn is_downstair(&self, path: &DungeonPath) -> bool {
        let address = Address::from_path(path);
        if address.level != self.level {
            return false;
        }
        if let Ok(cell) = self.current_floor.field.try_get_p(address.cd) {
            cell.surface == Surface::Stair
        } else {
            false
        }
    }
    fn level(&self) -> u32 {
        self.level
    }
    fn new_level(
        &mut self,
        game_info: &GameInfo,
        item: &mut ItemHandler,
        enemies: &mut EnemyHandler,
    ) -> GameResult<()> {
        self.new_level_(game_info, item, enemies, false)
    }
    fn can_move_player(&self, path: &DungeonPath, direction: Direction) -> Option<DungeonPath> {
        let address = Address::from_path(path);
        if address.level != self.level {
            return None;
        }
        self.current_floor
            .can_move_player(address.cd, direction)
            .map(|cd| DungeonPath::from(Address::new(address.level, cd)))
    }
    fn move_player(
        &mut self,
        path: &DungeonPath,
        direction: Direction,
        enemies: &mut EnemyHandler,
    ) -> GameResult<DungeonPath> {
        let address = Address::from_path(path);
        const ERR_STR: &str = "[rogue::Dungeon::move_player]";
        if address.level != self.level {
            bail!(ErrorKind::MaybeBug(ERR_STR));
        }
        self.current_floor.player_out(address.cd).context(ERR_STR)?;
        let cd = address.cd + direction.to_cd();
        let address = Address {
            level: self.level,
            cd,
        };
        self.current_floor
            .player_in(cd, false, enemies)
            .context(ERR_STR)?;
        Ok(address.into())
    }
    fn search(&mut self, path: &DungeonPath) -> GameResult<Vec<GameMsg>> {
        let address = Address::from_path(path);
        if address.level != self.level {
            bail!(ErrorKind::MaybeBug("[rogue::Dungeon::search]"));
        }
        Ok(self
            .current_floor
            .search(address.cd, &mut self.rng, &self.config)
            .collect())
    }
    fn select_cell(&mut self, is_character: bool) -> Option<DungeonPath> {
        self.current_floor
            .select_cell(&mut self.rng, is_character)
            .map(|cd| [self.level as i32, cd.x.0, cd.y.0].into())
    }
    fn enter_room(&mut self, path: &DungeonPath, enemies: &mut EnemyHandler) -> GameResult<()> {
        let address = Address::from_path(path);
        self.current_floor.player_in(address.cd, true, enemies)
    }
    fn draw(&self, drawer: &mut dyn FnMut(Positioned<Tile>) -> GameResult<()>) -> GameResult<()> {
        const ERR_STR: &str = "in rogue::Dungeon::move_player";
        let range = self
            .current_floor
            .field
            .size_ytrimed()
            .ok_or(ErrorKind::MaybeBug(ERR_STR))?;
        range.into_iter().try_for_each(|cd| {
            let cd = Coord::from(cd);
            let cell = self.current_floor.field.try_get_p(cd)?;
            drawer(Positioned(cd, cell.tile()))
        })
    }
    fn draw_ranges(&self) -> Vec<DungeonPath> {
        let xmax = self.config_global.width.0;
        let ymax = self.config_global.height.0 - 1;
        RectRange::from_ranges(0..xmax, 1..ymax)
            .unwrap()
            .into_iter()
            .filter(|&cd| self.current_floor.field.get_p(cd).is_obj_visible())
            .map(|cd| [self.level as i32, cd.0, cd.1].into())
            .collect()
    }
    fn path_to_cd(&self, path: &DungeonPath) -> Coord {
        Coord::new(path.0[1], path.0[2])
    }
    fn get_item(&self, path: &DungeonPath) -> Option<&ItemToken> {
        let addr = Address::from_path(path);
        if addr.level != self.level {
            return None;
        }
        self.current_floor.items.get(&addr.cd)
    }
    fn remove_item(&mut self, path: &DungeonPath) -> Option<ItemToken> {
        let addr = Address::from_path(path);
        if addr.level != self.level {
            return None;
        }
        if !self.current_floor.remove_obj(addr.cd, false) {
            return None;
        }
        self.current_floor.items.remove(&addr.cd)
    }
    fn tile(&mut self, path: &DungeonPath) -> Option<Tile> {
        let cd = self.path_to_cd(path);
        self.current_floor
            .field
            .try_get_mut_p(cd)
            .ok()
            .map(|s| s.tile())
    }
    fn get_history(&self, status: &PlayerStatus) -> Option<Array2<bool>> {
        let level = status.dungeon_level;
        if level == self.level {
            Some(self.current_floor.history_map())
        } else if let Some(floor) = self.past_floors.get(level as usize - 1) {
            Some(floor.history_map())
        } else {
            None
        }
    }
    fn move_enemy(
        &mut self,
        current: &DungeonPath,
        dist: &DungeonPath,
        skip: &dyn Fn(&DungeonPath) -> bool,
    ) -> MoveResult {
        let (cur, dist) = (current, dist).map(Address::from_path);
        if cur.level != dist.level {
            return MoveResult::CantMove;
        }
        let mut cand = Vec::new();
        let Dungeon {
            current_floor,
            dist_cache,
            ..
        } = self;
        let dist_map = dist_cache.make_dist_map(current_floor, dist.cd, true);
        for d in Direction::into_enum_iter() {
            let next = cur.cd + d.to_cd();
            if skip(&DungeonPath::from(Address::new(cur.level, next))) {
                continue;
            }
            let ndist = *dist_map.get_p(next);
            if ndist == 0 && current_floor.can_move_enemy(cur.cd, d) {
                return MoveResult::Reach;
            }
            if ndist != u32::max_value() && ndist > 0 {
                cand.push((ndist, next))
            }
        }
        if cand.is_empty() {
            return MoveResult::CantMove;
        }
        cand.sort_by_key(|t| t.0);
        let res = cand[0].1;
        MoveResult::CanMove(Address::new(cur.level, res).into())
    }
    fn move_enemy_randomly(
        &mut self,
        enemy_pos: &DungeonPath,
        player_pos: &DungeonPath,
        skip: &dyn Fn(&DungeonPath) -> bool,
    ) -> MoveResult {
        let cur = Address::from_path(enemy_pos);
        let idx = self.rng.range(0..8);
        let d = Direction::into_enum_iter().nth(idx).unwrap();
        let next = cur.cd + d.to_cd();
        if skip(&DungeonPath::from(Address::new(cur.level, next)))
            || !self.current_floor.can_move_enemy(cur.cd, d)
        {
            return MoveResult::CantMove;
        }
        let res = Address::new(cur.level, next).into();
        if res == *player_pos {
            MoveResult::Reach
        } else {
            MoveResult::CanMove(res)
        }
    }
    fn draw_enemy(&self, player: &DungeonPath, enemy: &DungeonPath) -> bool {
        let (p, e) = (player, enemy).map(Address::from_path);
        if p.level != e.level {
            return false;
        }
        p.cd.is_adjacent(e.cd) || self.current_floor.in_same_room(p.cd, e.cd)
    }
}

impl Dungeon {
    /// make new dungeon
    pub fn new(
        config: Config,
        config_global: &GlobalConfig,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
        enemies: &mut EnemyHandler,
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
            dist_cache: DistCache::new(),
        };
        dungeon
            .new_level_(game_info, item_handle, enemies, true)
            .context("rogue::Dungeon::new")?;
        Ok(dungeon)
    }

    fn new_level_(
        &mut self,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
        enemies: &mut EnemyHandler,
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
        let mut floor =
            Floor::gen_floor(level, &self.config, width, height, &mut self.rng).context(ERR_STR)?;
        debug!("[Dungeon::new_level] field: {}", floor.field);
        // setup gold
        let set_gold = !game_info.is_cleared || level >= self.max_level;
        debug!("[Dungeon::new_level] set_gold: {}", set_gold);
        floor.setup_items(level, item_handle, set_gold, &mut self.rng);
        // place stair
        floor.setup_stair(&mut self.rng).context(ERR_STR)?;
        // place enemies
        if !is_initial {
            enemies.remove_enemies();
        }
        floor.place_enemies(level, self.lev_add(), enemies, &mut self.rng);
        // place traps (STUB)
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

    fn lev_add(&self) -> u32 {
        if self.config.amulet_level < self.level {
            self.level - self.config.amulet_level
        } else {
            0
        }
    }
}

#[derive(Clone)]
struct DistCache {
    cache: VecDeque<(Array2<u32>, Coord)>,
}

impl DistCache {
    const MAX_CACHED_DIST: usize = 8;
    fn new() -> Self {
        DistCache {
            cache: VecDeque::with_capacity(Self::MAX_CACHED_DIST),
        }
    }
    fn make_dist_map(&mut self, floor: &Floor, cd: Coord, is_enemy: bool) -> &Array2<u32> {
        if let Some(pos) = self.cache.iter().position(|t| t.1 == cd) {
            return &self.cache[pos].0;
        }
        let dist_map = floor.make_dist_map(cd, is_enemy);
        let len = self.cache.len();
        self.cache.push_back((dist_map, cd));
        if len > Self::MAX_CACHED_DIST {
            self.cache.pop_front();
            &self.cache[len - 1].0
        } else {
            &self.cache[len].0
        }
    }
}

/// Address in the dungeon.
/// It's quite simple in rogue.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Address {
    /// level
    pub level: u32,
    /// coordinate
    pub cd: Coord,
}

impl Address {
    pub fn new(lev: u32, cd: Coord) -> Self {
        Address { level: lev, cd }
    }
    pub fn from_path(p: &DungeonPath) -> Self {
        Address {
            level: p[0] as u32,
            cd: Coord::new(p[1], p[2]),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Address, Coord, Direction, DungeonPath, MoveResult, TupleMap2};
    use crate::{GameConfig, RunTime};
    // tiny dungeon setting
    const CONFIG: &str = r#"
{
    "width": 32,
    "height": 16,
    "seed": 5,
    "dungeon": {
        "style": "rogue",
        "room_num_x": 2,
        "room_num_y": 2,
        "min_room_size": {
            "x": 4,
            "y": 4
        }
    }
}
"#;
    fn setup_runtime() -> RunTime {
        GameConfig::from_json(CONFIG).unwrap().build().unwrap()
    }
    #[test]
    fn test_move_enemy() {
        let mut runtime = setup_runtime();
        let mut check_move = |from, to, direc: Direction| {
            let next = from + direc.to_cd();
            let (from, to) = (from, to).map(|c| DungeonPath::from(Address::new(1, c)));
            assert_eq!(
                runtime.dungeon.move_enemy(&from, &to, &|_p| false),
                MoveResult::CanMove(Address::new(1, next).into())
            )
        };
        check_move(Coord::new(9, 9), Coord::new(28, 4), Direction::Right);
    }
}
