use super::{Room, Surface};
use dungeon::{Coord, Field};
use item::ItemRc;
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
