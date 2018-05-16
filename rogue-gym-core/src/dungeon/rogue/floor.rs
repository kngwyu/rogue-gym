use super::{passages, rooms, Config, Room, Surface};
use dungeon::{Cell, CellAttr, Coord, Direction, Field, Positioned, X, Y};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use fenwick::FenwickSet;
use item::ItemHandler;
use rect_iter::{Get2D, GetMut2D};
use rng::RngHandle;
use std::collections::HashSet;
/// representation of 'floor'
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Floor {
    /// rooms
    pub rooms: Vec<Room>,
    /// Coordinates of doors
    pub doors: HashSet<Coord>,
    /// field (level map)
    pub field: Field<Surface>,
    /// numbers of rooms which is not empty
    pub non_empty_rooms: FenwickSet,
}

impl Floor {
    fn new(rooms: Vec<Room>, doors: HashSet<Coord>, field: Field<Surface>) -> Self {
        let num_non_empty = rooms.iter().fold(0, |acc, room| {
            let plus = if room.is_empty() { 0 } else { 1 };
            acc + plus
        });
        Floor {
            rooms,
            doors,
            field,
            non_empty_rooms: FenwickSet::from_range(0..num_non_empty),
        }
    }

    /// generate a new floor without items
    pub fn gen_floor(
        level: u32,
        config: &Config,
        width: X,
        height: Y,
        rng: &mut RngHandle,
    ) -> GameResult<Self> {
        let rooms = rooms::gen_rooms(level, config, width, height, rng)
            .chain_err("dugeon::floor::Floor::new")?;
        let mut field = Field::new(width, height, Cell::with_default_attr(Surface::None));
        // in this phase, we can draw surfaces 'as is'
        rooms.iter().try_for_each(|room| {
            room.draw(|Positioned(cd, surface)| {
                field
                    .try_get_mut_p(cd)
                    .map(|mut_cell| {
                        mut_cell.surface = surface;
                        mut_cell.attr = attr_gen(surface, rng, level, config);
                    })
                    .into_chained("Floor::new")
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
                field
                    .try_get_mut_p(cd)
                    .map(|mut_cell| {
                        let attr = attr_gen(surface, rng, level, config);
                        // if the door is hiddden, don't draw it
                        if !mut_cell.attr.is_hidden() {
                            mut_cell.surface = surface;
                        }
                        if surface == Surface::Door {
                            doors.insert(cd);
                        }
                        mut_cell.attr = attr;
                    })
                    .into_chained("Floor::new dig_passges returned invalid index")
            })?;
        Ok(Floor::new(rooms, doors, field))
    }
    /// setup items for a floor
    pub fn setup_items(
        &mut self,
        level: u32,
        item_handle: &mut ItemHandler,
        set_gold: bool,
        rng: &mut RngHandle,
    ) -> GameResult<()> {
        // setup gold
        if set_gold {
            self.rooms
                .iter_mut()
                .filter(|room| !room.is_empty())
                .try_for_each(|room| {
                    item_handle.setup_gold(level, || {
                        let cd = room.select_cell(rng, false).ok_or_else(|| {
                            ErrorId::MaybeBug.into_with("rogue::Dungeon::setup_items")
                        })?;
                        room.fill_cell(cd, false);
                        Ok(vec![level as i32, cd.x.0, cd.y.0].into())
                    })
                })?;
        }
        Ok(())
    }
    fn can_move_impl(&self, cd: Coord, direction: Direction, is_enemy: bool) -> Option<bool> {
        let cur_cell = self.field.try_get_p(cd).ok()?;
        let nxt_cell = self.field.try_get_p(cd + direction.to_cd()).ok()?;
        // TODO: trap
        let mut res = match cur_cell.surface {
            Surface::Floor => match nxt_cell.surface {
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
        res &= !nxt_cell.attr.is_hidden();
        Some(res)
    }
    pub(crate) fn can_move_player(&self, cd: Coord, direction: Direction) -> bool {
        self.can_move_impl(cd, direction, false).unwrap_or(false)
    }
    fn cd_to_room_id(&self, cd: Coord) -> Option<usize> {
        self.rooms
            .iter()
            .enumerate()
            .find(|(_, room)| room.assigned_area.contains(cd))
            .map(|t| t.0)
    }
    pub(crate) fn enter_room(&mut self, cd: Coord) -> GameResult<()> {
        let room_num = match self.cd_to_room_id(cd) {
            Some(u) => u,
            None => {
                return Err(ErrorId::MaybeBug.into_with("[Floor::enter_room] no room for given cd"))
            }
        };
        let range = {
            let room = &mut self.rooms[room_num];
            if !room.is_normal() || room.is_visited || room.is_dark {
                room.is_visited = true;
                return Ok(());
            }
            room.range().unwrap().to_owned()
        };
        range.iter().try_for_each(|cd| {
            self.field
                .try_get_mut_p(cd)
                .map(|mut_cell| {
                    mut_cell.attr |= CellAttr::IS_DRAWN;
                    mut_cell.attr |= CellAttr::IS_VISIBLE;
                })
                .into_chained("Floor::enter_room")
        })
    }
    pub(crate) fn player_in(&mut self, cd: Coord, init: bool) -> GameResult<()> {
        if init || self.doors.contains(&cd) {
            self.enter_room(cd).chain_err("Floor::player_in")?;
        }
        self.field
            .try_get_mut_p(cd)
            .into_chained("Floor::player_in Cannot move")?
            .visit();
        Direction::iter_variants().take(4).for_each(|d| {
            let cd = cd + d.to_cd();
            if let Ok(cell) = self.field.try_get_mut_p(cd) {
                cell.approached_by_player();
            }
        });
        Ok(())
    }

    // STUB
    pub(crate) fn player_out(&mut self, cd: Coord) -> GameResult<()> {
        Ok(())
    }

    pub(crate) fn fill_cell(&mut self, cd: Coord, is_character: bool) -> bool {
        fn fill_cell_impl(rooms: &mut Vec<Room>, cd: Coord, is_character: bool) -> Option<bool> {
            let room = rooms.iter_mut().find(|room| room.contains(cd))?;
            Some(room.fill_cell(cd, is_character))
        }
        fill_cell_impl(&mut self.rooms, cd, is_character) == Some(true)
    }
    pub(crate) fn select_cell(&self, rng: &mut RngHandle, is_character: bool) -> Option<Coord> {
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
}

// generate initial attribute of cell
fn attr_gen(surface: Surface, rng: &mut RngHandle, level: u32, config: &Config) -> CellAttr {
    let mut attr = CellAttr::default();
    match surface {
        Surface::Passage => {
            if rng.range(0..config.dark_level) + 1 < level
                && rng.does_happen(config.hidden_passage_rate_inv)
            {
                attr |= CellAttr::IS_HIDDEN;
            }
        }
        Surface::Door => {
            if rng.range(0..config.dark_level) + 1 < level
                && rng.does_happen(config.locked_door_rate_inv)
            {
                attr |= CellAttr::IS_LOCKED;
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
                assert!(floor.fill_cell(cd, false));
            } else {
                break;
            }
        }
        assert!(cnt > 15);
    }
}
