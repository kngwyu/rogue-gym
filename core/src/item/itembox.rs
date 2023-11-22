//! utility for managing character's items
use log::debug;

use super::{Item, ItemToken};
use crate::fenwick::FenwickSet;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ItemBox {
    empty_chars: FenwickSet,
    items: BTreeMap<usize, ItemToken>,
}

impl ItemBox {
    pub fn with_capacity(max_len: usize) -> Self {
        ItemBox {
            empty_chars: FenwickSet::from_range(0..max_len),
            items: BTreeMap::new(),
        }
    }
    pub fn add(&mut self, item: ItemToken) -> bool {
        let ch = match self.empty_chars.nth(0) {
            Some(id) => id,
            None => return false,
        };
        debug!("ItemBox::add {}", ch);
        self.insert(ch, item);
        true
    }
    pub fn entry(&mut self, token: &ItemToken) -> Option<Entry> {
        if token.is_many() {
            if let Some(merge_id) = self.check_merge(token.get()) {
                return Some(Entry::Merge(MergeEntry(
                    self.items.get_mut(&merge_id).unwrap(),
                )));
            }
        }
        let insert_pos = self.empty_chars.nth(0)?;
        Some(Entry::Insert(InsertEntry(insert_pos, self)))
    }
    pub fn tokens(&self) -> impl Iterator<Item = &ItemToken> {
        self.items.values()
    }
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.tokens().map(|t| t.get())
    }
    pub fn find_by(&self, mut query: impl FnMut(&Item) -> bool) -> Option<&ItemToken> {
        self.items
            .iter()
            .find(|(_, item)| query(item.get()))
            .map(|(_, i)| i)
    }
    fn check_merge(&self, got_item: &Item) -> Option<usize> {
        // check if we can merge item or not
        self.items
            .iter()
            .find(|(_, token)| token.get().kind == got_item.kind)
            .map(|t| *t.0)
    }
    fn insert(&mut self, ch: usize, item: ItemToken) {
        self.items.insert(ch, item);
        self.empty_chars.remove(ch);
    }
}

#[derive(Debug)]
pub enum Entry<'a> {
    Merge(MergeEntry<'a>),
    Insert(InsertEntry<'a>),
}

#[derive(Debug)]
pub struct MergeEntry<'a>(&'a mut ItemToken);

impl<'a> MergeEntry<'a> {
    pub fn exec(self, item: Item) -> Item {
        self.0.get_mut().merge(item.clone(), |a, b| a | b);
        item
    }
}

#[derive(Debug)]
pub struct InsertEntry<'a>(usize, &'a mut ItemBox);

impl<'a> InsertEntry<'a> {
    pub fn exec(self, item: ItemToken) -> Item {
        let res = item.get_cloned();
        self.1.insert(self.0, item);
        res
    }
}
