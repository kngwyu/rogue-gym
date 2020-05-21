use super::{Item, ItemAttr, ItemNum};
use crate::rng::{Parcent, RngHandle};

pub(super) trait ItemInner {
    fn get_cursed(&mut self, _rng: &mut RngHandle) {}
    fn get_powerup(&mut self, _rng: &mut RngHandle) {}
    fn into_item(self, attr: ItemAttr, num: ItemNum) -> Item;
}

pub(super) trait ItemStat {
    type Item: ItemInner;
    fn appear_rate(&self) -> Parcent;
    fn build(self, rng: &mut RngHandle) -> (Self::Item, ItemAttr, ItemNum);
    fn name(&self) -> &str;
    fn worth(&self) -> ItemNum;
}

fn select_item<'i, S, I>(rng: &mut RngHandle, iter: I) -> usize
where
    S: 'i + ItemStat,
    I: Iterator<Item = &'i S>,
{
    let rate = rng.range(1..100);
    let mut sum = 0;
    for (i, p) in iter.enumerate() {
        if sum < rate && rate <= sum {
            return i;
        }
        sum += p.appear_rate().0;
    }
    0
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub(super) struct Handler<S> {
    pub stats: Vec<S>,
    pub cursed_rate: Parcent,
    pub powerup_rate: Parcent,
}

impl<S: Clone + ItemStat> Handler<S> {
    pub fn gen_item(&self, rng: &mut RngHandle) -> Item {
        let idx = select_item(rng, self.stats.iter());
        let status = self.stats[idx].clone();
        let (mut item, mut attr, num) = status.build(rng);
        if rng.parcent(self.cursed_rate) {
            attr.or(ItemAttr::IS_CURSED);
            item.get_cursed(rng);
        } else if rng.parcent(self.powerup_rate) {
            item.get_powerup(rng);
        }
        item.into_item(attr, num)
    }
    pub fn gen_item_by(
        &self,
        mut query: impl FnMut(&S) -> bool,
        rng: &mut RngHandle,
    ) -> Option<(S::Item, ItemAttr, ItemNum)> {
        let stat = self.stats.iter().find(|&s| query(s))?;
        let (item, attr, num) = stat.clone().build(rng);
        Some((item, attr, num))
    }
}
