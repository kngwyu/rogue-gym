use super::{passages, rooms, Config, Room, Surface};
use dungeon::{Cell, CellAttr, Coord, Direction, Field, Positioned, X, Y};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use item::ItemHandler;
use item::ItemRc;
use rect_iter::{Get2D, GetMut2D};
use rng::RngHandle;
use std::collections::{HashMap, HashSet};
/// representation of 'floor'
#[derive(Clone, Debug, Default)]
pub struct Floor {
    /// rooms
    pub rooms: Vec<Room>,
    /// numbers of rooms which is not empty
    pub nonempty_rooms: usize,
    /// items set in the floor
    pub items: HashMap<Coord, ItemRc>,
    /// Coordinates of doors
    pub doors: HashSet<Coord>,
    /// field (level map)
    pub field: Field<Surface>,
}

impl Floor {
    /// generate a new floor without items
    pub fn with_no_item(
        level: u32,
        config: &Config,
        width: X,
        height: Y,
        rng: &mut RngHandle,
    ) -> GameResult<Self> {
        let rooms = rooms::gen_rooms(level, config, width, height, rng)
            .chain_err("[dugeon::floor::Floor::new]")?;
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
                    .into_chained("[Floor::new]")
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
                    .into_chained("[Floor::new] dig_passges returned invalid index")
            })?;
        let nonempty_rooms = rooms.iter().fold(0, |acc, room| {
            let plus = if room.is_empty() { 0 } else { 1 };
            acc + plus
        });
        Ok(Floor {
            rooms,
            nonempty_rooms,
            items: HashMap::new(),
            doors,
            field,
        })
    }
    /// setup items for a floor
    pub fn setup_items(
        &mut self,
        level: u32,
        item_handle: &mut ItemHandler,
        set_gold: bool,
        rng: &mut RngHandle,
    ) {
        let mut items = HashMap::new();
        // setup gold
        if set_gold {
            self.rooms
                .iter_mut()
                .filter(|room| !room.is_empty())
                .for_each(|room| {
                    item_handle.setup_gold(level, |item_rc| {
                        if let Some(cell) = room.select_empty_cell(rng) {
                            items.insert(cell, item_rc);
                            room.fill_cell(cell);
                        }
                    })
                });
        }
        self.items = items;
    }
    fn can_move_impl(&self, cd: Coord, direction: Direction, is_enemy: bool) -> GameResult<bool> {
        let cur_cell = self.field.try_get_p(cd).into_chained("")?;
        let nxt_cell = self.field
            .try_get_p(cd + direction.to_cd())
            .into_chained("")?;
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
        Ok(res)
    }
    pub(crate) fn can_move_player(&self, cd: Coord, direction: Direction) -> bool {
        if let Ok(b) = self.can_move_impl(cd, direction, false) {
            b
        } else {
            false
        }
    }
    fn cd_to_room_id(&self, cd: Coord) -> Option<usize> {
        self.rooms
            .iter()
            .enumerate()
            .find(|(_, room)| room.assigned_area.contains(cd))
            .map(|t| t.0)
    }
    fn enter_room(&mut self, cd: Coord) -> GameResult<()> {
        let room_num = match self.cd_to_room_id(cd) {
            Some(u) => u,
            None => {
                return Err(ErrorId::LogicError.into_with("[Floor::enter_room] no room for given cd"))
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
            if range.is_edge(cd) {
                return Ok(());
            }
            self.field
                .try_get_mut_p(cd)
                .map(|mut_cell| {
                    mut_cell.attr |= CellAttr::IS_DRAWN;
                    mut_cell.attr |= CellAttr::IS_VISIBLE;
                })
                .into_chained("[Floor::enter_room]")
        })
    }
    pub(crate) fn move_player(&mut self, cd: Coord) -> GameResult<()> {
        if self.doors.contains(&cd) {
            self.enter_room(cd).chain_err("[Floor::move_player]")?;
        }
        if let Ok(cell) = self.field.try_get_mut_p(cd) {
            cell.attr |= CellAttr::IS_VISITED;
        }
        Ok(())
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

mod test {
    use super::*;
    use item::ItemConfig;
    use rect_iter::{Get2D, RectRange};
    use rng::Rng;
    use tile::Drawable;
    #[test]
    #[ignore]
    fn print_floor() {
        let config = Config::default();
        let mut rng = RngHandle::new();
        let floor = Floor::with_no_item(10, &config, X(80), Y(24), &mut rng).unwrap();
        println!("{}", floor.field);
    }
    #[test]
    #[ignore]
    fn print_item_floor() {
        let config = Config::default();
        let mut rng = RngHandle::new();
        let mut floor = Floor::with_no_item(10, &config, X(80), Y(24), &mut rng).unwrap();
        let item_config = ItemConfig::default();
        let mut rng = RngHandle::new();
        let mut item_handle = ItemHandler::new(item_config, rng.gen());
        floor.setup_items(10, &mut item_handle, true, &mut rng);
        println!("{}", floor.field);
        RectRange::zero_start(80, 24)
            .unwrap()
            .into_iter()
            .for_each(|cd| {
                let cd: Coord = cd.into();
                let tile = if let Some(item) = floor.items.get(&cd) {
                    item.tile()
                } else {
                    floor.field.get_p(cd).surface.tile()
                };
                print!("{}", tile);
                if cd.x == X(79) {
                    println!("");
                }
            });
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
                let floor = Floor::with_no_item(i, &config, X(w), Y(h), &mut rng).unwrap();
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
}
