//! utility for managing character's items
use super::{Item, ItemHandler, ItemId};
use error::{ErrorId, ErrorKind, GameResult};
use fenwick::FenwickSet;
use std::collections::{btree_map, BTreeMap};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemPack {
    empty_chars: FenwickSet,
    items: BTreeMap<usize, ItemId>,
}

impl ItemPack {
    pub fn from_max_len(max_len: usize) -> Self {
        ItemPack {
            empty_chars: FenwickSet::from_range(0..max_len),
            items: BTreeMap::new(),
        }
    }
    pub fn add(&mut self, id: ItemId) -> bool {
        let ch = match self.empty_chars.nth(0) {
            Some(id) => id,
            None => return false,
        };
        self.empty_chars.insert(ch);
        self.items.insert(ch, id);
        true
    }
    pub fn entry(&self, id: ItemId, handle: &ItemHandler) -> Option<PackEntry> {
        let got_item = handle.get(id)?;
        if got_item.is_many() {
            if let Some(merge) = self.check_merge(got_item, handle) {
                return Some(PackEntry::Merge(merge));
            }
        }
        self.empty_chars
            .nth(0)
            .map(|u| PackEntry::Insert(InsertEntry(u)))
    }
    pub fn ids(&self) -> btree_map::Values<usize, ItemId> {
        self.items.values()
    }
    fn check_merge(&self, got_item: &Item, handle: &ItemHandler) -> Option<MergeEntry> {
        // check if we can merge item or not
        self.items
            .iter()
            .find(|(_, &id)| {
                if let Some(item) = handle.get(id) {
                    item.kind == got_item.kind
                } else {
                    false
                }
            })
            .map(|(_, &id)| MergeEntry(id))
    }
    fn insert(&mut self, ch: usize, id: ItemId) {
        self.items.insert(ch, id);
        self.empty_chars.remove(ch);
    }
}

#[derive(Clone, Debug)]
pub enum PackEntry {
    Merge(MergeEntry),
    Insert(InsertEntry),
}

#[derive(Clone, Copy, Debug)]
pub struct MergeEntry(ItemId);

impl MergeEntry {
    pub fn exec(self, handler: &mut ItemHandler, got_item: Item) -> GameResult<()> {
        handler
            .get_mut(self.0)
            .map(|item| item.merge(got_item, |a, b| a | b))
            .ok_or_else(|| {
                ErrorId::MaybeBug.into_with(|| "[item::pack::MergeEntry::exec] can't pack item")
            })
    }
    pub fn id(self) -> ItemId {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InsertEntry(usize);

impl InsertEntry {
    pub fn exec(self, pack: &mut ItemPack, id: ItemId) {
        pack.insert(self.0, id);
    }
}
