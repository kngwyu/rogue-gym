//! rogue floor
use super::{passages, rooms, Address, Config, Room, Surface};
use dungeon::{Cell, CellAttr, Coord, Direction, Field, Positioned, X, Y};
use enemies::EnemyHandler;
use enum_iterator::IntoEnumIterator;
use error::*;
use fenwick::FenwickSet;
use item::{ItemHandler, ItemToken};
use ndarray::Array2;
use rect_iter::{Get2D, GetMut2D};
use rng::RngHandle;
use std::collections::{HashMap, HashSet, VecDeque};
use GameMsg;

/// representation of 'floor'
#[derive(Clone, Debug, Default)]
pub struct Floor {
    /// rooms
    pub rooms: Vec<Room>,
    /// Coordinates of doors
    pub doors: HashSet<Coord>,
    /// field (level map)
    pub field: Field<Surface>,
    /// ids of rooms which are not empty
    pub non_empty_rooms: FenwickSet,
    /// items
    pub items: HashMap<Coord, ItemToken>,
}

impl Floor {
    fn new(rooms: Vec<Room>, doors: HashSet<Coord>, field: Field<Surface>) -> Self {
        let non_empty_rooms =
            rooms
                .iter()
                .fold(FenwickSet::with_capacity(rooms.len()), |mut s, room| {
                    if !room.is_empty() {
                        s.insert(room.id);
                    }
                    s
                });
        Floor {
            rooms,
            doors,
            field,
            non_empty_rooms,
            items: Default::default(),
        }
    }

    /// generate a new floor without items
    // TODO: trap
    pub fn gen_floor(
        level: u32,
        config: &Config,
        width: X,
        height: Y,
        rng: &mut RngHandle,
    ) -> GameResult<Self> {
        let rooms = rooms::gen_rooms(level, config, width, height, rng)
            .chain_err(|| "Error in gen_floor")?;
        let mut field = Field::new(width, height, Cell::with_default_attr(Surface::None));
        // in this phase, we can draw surfaces 'as is'
        rooms.iter().try_for_each(|room| {
            room.draw(|Positioned(cd, surface)| {
                field
                    .try_get_mut_p(cd)
                    .map(|mut_cell| {
                        mut_cell.surface = surface;
                        mut_cell.attr = gen_attr(surface, room.is_dark, rng, level, config);
                    })
                    .into_chained(|| "Error in gen_floor")
            })
        })?;
        // sometimes door is hidden randomly so first we store positions to avoid borrow restriction
        let mut passages = Vec::new();
        passages::dig_passges(
            &rooms,
            config.room_num_x,
            config.room_num_y,
            rng,
            config.max_extra_edges,
            |p| {
                passages.push(p);
                Ok(())
            },
        )?;
        let mut doors = HashSet::new();
        passages
            .into_iter()
            .try_for_each(|Positioned(cd, surface)| {
                if surface == Surface::Door {
                    doors.insert(cd);
                }
                field
                    .try_get_mut_p(cd)
                    .map(|cell| {
                        cell.attr = gen_attr(surface, false, rng, level, config);
                        // if the passage is not hiddden, let's draw
                        if !cell.is_hidden() && !cell.is_locked() {
                            cell.surface = surface;
                        }
                    })
                    .into_chained(|| "Floor::new dig_passges returned invalid index")
            })?;
        Ok(Floor::new(rooms, doors, field))
    }
    /// place enemies
    pub fn place_enemies(
        &mut self,
        level: u32,
        lev_add: u32,
        enemies: &mut EnemyHandler,
        rng: &mut RngHandle,
    ) {
        // For backward compatibility
        if enemies.is_no_enemy() {
            return;
        }
        let min = level.checked_sub(4).unwrap_or(0);
        let max = level + 6;
        for (cd, room) in self
            .rooms
            .iter_mut()
            .filter_map(|room| Some((room.select_cell(rng, true)?, room)))
        {
            if let Some(enemy) = enemies.gen_enemy(min..max, i64::from(lev_add), room.has_gold) {
                let place = Address::new(level, cd).into();
                enemies.place(place, enemy);
                room.fill_cell(cd, true);
            }
        }
    }
    /// setup items for a floor
    pub fn setup_items(
        &mut self,
        level: u32,
        item_handle: &mut ItemHandler,
        set_gold: bool,
        rng: &mut RngHandle,
    ) {
        // setup gold
        if set_gold {
            for (cd, room) in self
                .rooms
                .iter_mut()
                .filter_map(|room| Some((room.select_cell(rng, false)?, room)))
            {
                if let Some(gold) = item_handle.setup_gold(level) {
                    room.fill_cell(cd, false);
                    room.has_gold = true;
                    self.items.insert(cd, gold);
                }
            }
        }
    }

    /// set stair
    pub fn setup_stair(&mut self, rng: &mut RngHandle) -> GameResult<()> {
        let cd = self
            .select_cell(rng, false)
            .ok_or_else(|| ErrorId::MaybeBug.into_with(|| "[setup stair] no empty cell!"))?;
        {
            let cell = self
                .field
                .try_get_mut_p(cd)
                .into_chained(|| "[setup stair] select_cell returned invalid coord")?;
            cell.surface = Surface::Stair;
        }
        self.set_obj(cd, false);
        Ok(())
    }

    fn can_move_impl(&self, cd: Coord, direction: Direction, is_enemy: bool) -> Option<bool> {
        let cur_cell = self.field.try_get_p(cd).ok()?;
        let nxt_cell = self.field.try_get_p(cd + direction.to_cd()).ok()?;

        // TODO: trap
        let mut res = match cur_cell.surface {
            Surface::Floor | Surface::Stair => match nxt_cell.surface {
                Surface::Floor | Surface::Stair | Surface::Trap => true,
                Surface::Door | Surface::Passage => !direction.is_diag(),
                _ => false,
            },
            Surface::Passage => match nxt_cell.surface {
                Surface::Passage | Surface::Stair | Surface::Trap | Surface::Door => {
                    !direction.is_diag() || is_enemy
                }
                _ => false,
            },
            Surface::Door => match nxt_cell.surface {
                Surface::Passage
                | Surface::Stair
                | Surface::Trap
                | Surface::Door
                | Surface::Floor => !direction.is_diag(),
                _ => false,
            },
            _ => false,
        };
        res &= nxt_cell.surface.can_walk();
        res &= !nxt_cell.is_hidden();
        res &= !nxt_cell.is_locked();
        Some(res)
    }

    /// judge if the player can move from `cd` in `direction`
    crate fn can_move_player(&self, cd: Coord, direction: Direction) -> bool {
        self.can_move_impl(cd, direction, false).unwrap_or(false)
    }

    /// judge if the enemy can move from `cd` in `direction`
    crate fn can_move_enemy(&self, cd: Coord, direction: Direction) -> bool {
        self.can_move_impl(cd, direction, true).unwrap_or(false)
    }

    fn cd_to_room_id(&self, cd: Coord) -> Option<usize> {
        self.rooms
            .iter()
            .enumerate()
            .find(|(_, room)| room.assigned_area.contains(cd))
            .map(|t| t.0)
    }

    fn with_current_room<S, M>(&mut self, cd: Coord, select: S, mut mark: M) -> GameResult<()>
    where
        S: FnOnce(&mut Room) -> bool,
        M: FnMut(&mut Cell<Surface>, /* is_edge: */ bool),
    {
        let room_id = match self.cd_to_room_id(cd) {
            Some(u) => u,
            None => {
                return Err(ErrorId::MaybeBug
                    .into_with(|| "[Floor::with_current_room] no room for given coord"))
            }
        };
        if !select(&mut self.rooms[room_id]) {
            return Ok(());
        }
        let range = self.rooms[room_id]
            .range()
            .unwrap_or(&self.rooms[room_id].assigned_area)
            .to_owned();
        range.iter().try_for_each(|cd| {
            let is_edge = range.is_edge(cd);
            self.field
                .try_get_mut_p(cd)
                .map(|mut_cell| mark(mut_cell, is_edge))
                .into_chained(|| "in Floor::with_current_room")
        })
    }

    /// player enters room
    crate fn enters_room(&mut self, cd: Coord) -> GameResult<()> {
        self.with_current_room(
            cd,
            |room| {
                if room.is_visited {
                    return false;
                }
                room.is_visited = true;
                room.is_normal() && !room.is_dark
            },
            |cell, _| {
                cell.attr |= CellAttr::HAS_DRAWN;
                cell.visible(true);
            },
        )
        .chain_err(|| "Floor::enters_room")
    }

    /// player enters room
    crate fn leaves_room(&mut self, cd: Coord) -> GameResult<()> {
        self.with_current_room(
            cd,
            |room| room.is_visited && room.is_dark,
            |cell, is_edge| {
                if !is_edge {
                    cell.visible(false);
                }
            },
        )
        .chain_err(|| "Floor::leaves_room")
    }

    /// player walks in the cell
    crate fn player_in(
        &mut self,
        cd: Coord,
        init: bool,
        level: u32,
        enemies: &mut EnemyHandler,
    ) -> GameResult<()> {
        debug!("[Floor::player_in] cd: {:?}", cd);
        if init || self.doors.contains(&cd) {
            self.enters_room(cd).chain_err(|| "Floor::player_in")?;
            if let Some(room_id) = self.cd_to_room_id(cd) {
                let room = &self.rooms[room_id];
                enemies.activate(|p| {
                    let cd = Address::from_path(p).cd;
                    room.assigned_area.contains(cd)
                });
            }
        }
        self.field
            .try_get_mut_p(cd)
            .into_chained(|| "Floor::player_in Cannot move")?
            .visit();
        self.set_obj(cd, true);
        Direction::into_enum_iter().take(9).for_each(|d| {
            let cd = cd + d.to_cd();
            if let Ok(cell) = self.field.try_get_mut_p(cd) {
                if !d.is_diag() || cell.surface != Surface::Passage {
                    cell.approached();
                }
            }
        });
        Ok(())
    }

    /// player leaves the cell
    crate fn player_out(&mut self, cd: Coord) -> GameResult<()> {
        if self.doors.contains(&cd) {
            self.leaves_room(cd).chain_err(|| "Floor::player_out")?;
        }
        self.remove_obj(cd, true);
        Direction::into_enum_iter().take(9).for_each(|d| {
            let cd = cd + d.to_cd();
            if let Ok(cell) = self.field.try_get_mut_p(cd) {
                if cell.surface == Surface::Floor {
                    cell.left();
                }
            }
        });
        Ok(())
    }

    /// register an object to cell
    crate fn set_obj(&mut self, cd: Coord, is_character: bool) -> bool {
        let mut impl_ = || {
            let room = self.rooms.iter_mut().find(|room| room.contains(cd))?;
            Some(room.fill_cell(cd, is_character))
        };
        impl_() == Some(true)
    }

    /// unregister an object to cell
    crate fn remove_obj(&mut self, cd: Coord, is_character: bool) -> bool {
        let mut impl_ = || {
            let room = self.rooms.iter_mut().find(|room| room.contains(cd))?;
            Some(room.unfill_cell(cd, is_character))
        };
        impl_() == Some(true)
    }

    /// select an empty cell from rooms randomly
    crate fn select_cell(&self, rng: &mut RngHandle, is_character: bool) -> Option<Coord> {
        let mut candidates = self.non_empty_rooms.clone();
        while candidates.len() > 0 {
            let room_idx = candidates
                .select(rng)
                .expect("Logic Error in floor::select_cell");
            if let Some(cd) = self.rooms[room_idx].select_cell(rng, is_character) {
                return Some(cd);
            } else {
                candidates.remove(room_idx);
            }
        }
        None
    }

    /// search command
    crate fn search<'a>(
        &'a mut self,
        cd: Coord,
        rng: &'a mut RngHandle,
        config: &'a Config,
    ) -> impl 'a + Iterator<Item = GameMsg> {
        let probinc = 0; // TODO: it should be changed by player status
        Direction::into_enum_iter().take(8).filter_map(move |d| {
            let cd = cd + d.to_cd();
            let cell = self.field.try_get_mut_p(cd).ok()?;
            if cell.is_hidden() && rng.does_happen(probinc + config.passage_unlock_rate_inv) {
                cell.unlock();
                cell.surface = Surface::Passage;
            }
            if cell.is_locked() && rng.does_happen(probinc + config.door_unlock_rate_inv) {
                cell.unlock();
                cell.surface = Surface::Door;
                return Some(GameMsg::SecretDoor);
            }
            None
        })
    }

    crate fn history_map(&self) -> Array2<bool> {
        let size = self.field.size();
        let mut array = Array2::from_elem([size.ylen() as usize, size.xlen() as usize], false);
        size.into_iter().for_each(|cd| {
            *array.get_mut_p(cd) = self.field.get_p(cd).is_visited();
        });
        array
    }

    crate fn make_dist_map(&self, from: Coord, is_enemy: bool) -> Array2<u32> {
        let (w, h) = (self.field.width(), self.field.height());
        let inf = u32::max_value();
        let mut dist = Array2::from_elem([h.0 as usize, w.0 as usize], inf);
        let mut queue = VecDeque::new();
        *dist.get_mut_p(from) = 0;
        queue.push_back(from);
        while let Some(current) = queue.pop_front() {
            for d in Direction::into_enum_iter().take(8) {
                let next = current + d.to_cd();
                let cdist = *dist.get_p(current);
                if let Ok(ndist) = dist.try_get_mut_p(next) {
                    if *ndist != inf || self.can_move_impl(current, d, is_enemy) != Some(true) {
                        continue;
                    }
                    queue.push_back(next);
                    *ndist = cdist + 1;
                }
            }
        }
        dist
    }
}

// generate initial attribute of cell
fn gen_attr(
    surface: Surface,
    is_dark: bool,
    rng: &mut RngHandle,
    level: u32,
    config: &Config,
) -> CellAttr {
    let mut attr = CellAttr::default();
    match surface {
        Surface::Passage => {
            if rng.range(..config.dark_level) < level
                && rng.does_happen(config.hidden_passage_rate_inv)
            {
                attr |= CellAttr::IS_HIDDEN;
            }
        }
        Surface::Door => {
            if rng.range(..config.dark_level) < level
                && rng.does_happen(config.locked_door_rate_inv)
            {
                attr |= CellAttr::IS_LOCKED;
            }
        }
        Surface::Floor => {
            if is_dark {
                attr |= CellAttr::IS_DARK
            }
        }
        _ => {}
    }
    attr
}

#[cfg(test)]
mod test {
    use super::*;
    use rect_iter::RectRange;
    #[test]
    #[ignore]
    fn print_floor() {
        let config = Config::default();
        let mut rng = RngHandle::new();
        let floor = Floor::gen_floor(10, &config, X(80), Y(24), &mut rng).unwrap();
        println!("{}", floor.field);
    }
    #[test]
    fn secret_door() {
        let config = Config::default();
        let mut rng = RngHandle::new();
        let (w, h) = (80, 24);
        let mut before = 0;
        for i in 1..10 {
            let mut hidden = 0;
            for _ in 0..100 {
                let floor = Floor::gen_floor(i, &config, X(w), Y(h), &mut rng).unwrap();
                hidden += RectRange::zero_start(w, h)
                    .unwrap()
                    .into_iter()
                    .filter(|&cd| {
                        let cd: Coord = cd.into();
                        let cell = floor.field.get_p(cd);
                        cell.surface != Surface::Door && floor.doors.contains(&cd)
                    })
                    .count();
            }
            assert!(before <= hidden + 10);
            before = hidden;
        }
    }
    #[test]
    fn select_cell() {
        let config = Config::default();
        let mut rng = RngHandle::new();
        let (w, h) = (80, 24);
        let mut floor = Floor::gen_floor(10, &config, X(w), Y(h), &mut rng).unwrap();
        let mut cnt = 0;
        for _ in 0..1000 {
            let cd = floor.select_cell(&mut rng, false);
            if let Some(cd) = cd {
                cnt += 1;
                assert!(floor.set_obj(cd, false));
            } else {
                break;
            }
        }
        assert!(cnt > 15);
    }
}
