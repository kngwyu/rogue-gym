#[macro_export]
macro_rules! item_stat {
    ($name: ty) => {
        impl crate::item::ItemStat for $name {
            fn appear_rate(&self) -> Parcent {
                self.appear_rate
            }
            fn worth(&self) -> crate::item::ItemNum {
                self.worth
            }
        }
    };
}
