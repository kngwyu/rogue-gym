use super::{passages, rooms, Config, Room, Surface};
use dungeon::{Cell, CellAttr, Coord, Field, Positioned, X, Y};
use error::{GameResult, ResultExt};
use item::ItemRc;
use rect_iter::{Get2D, GetMut2D};
use rng::RngHandle;
use std::collections::BTreeMap;
/// representation of 'floor'
#[derive(Clone, Debug)]
pub struct Floor {
    /// rooms
    rooms: Vec<Room>,
    /// items
    item_map: BTreeMap<Coord, ItemRc>,
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
        // TODO: can't generate cell attribute here
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
        Ok(Floor {
            rooms,
            item_map: BTreeMap::new(),
            field,
        })
    }
}

// STUB!!!
fn attr_gen(surface: Surface, rng: &mut RngHandle) -> CellAttr {
    CellAttr::default()
}

mod test {
    use super::*;
    #[test]
    #[ignore]
    fn print_floor() {
        let config = Config::default();
        let mut rng = RngHandle::new();
        let floor = Floor::with_no_item(10, &config, X(80), Y(24), &mut rng).unwrap();
        println!("{}", floor.field);
    }
}
