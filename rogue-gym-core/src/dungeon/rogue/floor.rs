use super::{passages, rooms, Config, Room, Surface};
use dungeon::{Cell, CellAttr, Coord, Field, Positioned, X, Y};
use error::{GameResult, ResultExt};
use item::ItemHandler;
use item::ItemRc;
use rect_iter::GetMut2D;
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
    field: Field<Surface>,
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
                        if !(attr.contains(CellAttr::IS_HIDDEN) && surface == Surface::Door) {
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
}

// generate initial attribute of cell
fn attr_gen(surface: Surface, rng: &mut RngHandle, level: u32, config: &Config) -> CellAttr {
    let mut attr = CellAttr::default();
    match surface {
        Surface::Passage => {
            if rng.range(0..config.dark_level) + 1 < level
                && rng.does_happen(config.hidden_rate_inv)
            {
                attr |= CellAttr::IS_HIDDEN;
            }
        }
        Surface::Door => {
            if rng.range(0..config.dark_level) + 1 < level
                && rng.does_happen(config.door_lock_rate_inv)
            {
                attr |= CellAttr::IS_HIDDEN;
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
    use Drawable;
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
