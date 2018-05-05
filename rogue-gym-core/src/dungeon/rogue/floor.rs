use super::{passages, rooms, Config, Room, Surface};
use dungeon::{Cell, CellAttr, Coord, Field, Positioned, X, Y};
use error::{GameResult, ResultExt};
use item::ItemHandler;
use item::ItemRc;
use rect_iter::GetMut2D;
use rng::RngHandle;
use std::collections::HashMap;
use GameInfo;
/// representation of 'floor'
#[derive(Clone, Debug)]
pub struct Floor {
    /// rooms
    pub rooms: Vec<Room>,
    /// numbers of rooms which is not empty
    pub nonempty_rooms: usize,
    pub items: HashMap<Coord, ItemRc>,
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
        rooms.iter().try_for_each(|room| {
            room.draw(|Positioned(cd, surface)| {
                field
                    .try_get_mut_p(cd)
                    .map(|mut_cell| {
                        mut_cell.surface = surface;
                        mut_cell.attr = attr_gen(surface, rng);
                    })
                    .into_chained("[Floor::new]")
            })
        })?;
        // NOTE: can't generate cell attribute here
        passages::dig_passges(
            &rooms,
            config.room_num_x,
            config.room_num_y,
            rng,
            config.max_extra_edges,
            |Positioned(cd, surface)| {
                field
                    .try_get_mut_p(cd)
                    .map(|mut_cell| {
                        mut_cell.surface = surface;
                    })
                    .into_chained("[Floor::new]")
            },
        )?;
        let nonempty_rooms = rooms.iter().fold(0, |acc, room| {
            let plus = if room.is_empty() { 0 } else { 1 };
            acc + plus
        });
        Ok(Floor {
            rooms,
            items: HashMap::new(),
            field,
            nonempty_rooms,
        })
    }
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

// STUB!!!
fn attr_gen(surface: Surface, rng: &mut RngHandle) -> CellAttr {
    CellAttr::default()
}

mod test {
    use super::*;
    use item::ItemConfig;
    use rect_iter::{Get2D, RectRange};
    use rng::Rng;
    use Tile;
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
    fn print_floor_with_item() {
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
                let byte = if let Some(item) = floor.items.get(&cd) {
                    item.byte()
                } else {
                    floor.field.get_p(cd).surface.byte()
                };
                print!("{}", byte as char);
                if cd.x == X(79) {
                    println!("");
                }
            });
    }
}
