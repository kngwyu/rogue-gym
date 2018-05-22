pub mod player;
use rng::RngHandle;

pub use self::player::{Action, Hunger, Leveling, Player};
/// values compatible with Hit Point
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Ord, Eq, Add, Sub, Mul, Div,
         Neg, AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct HitPoint(i64);

/// values compatible with strength
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Ord, Eq, Add, Sub, Mul, Div,
         Neg, AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct Strength(i64);

/// values compatible with defense power of Armors
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Ord, Eq, Add, Sub, Mul, Div,
         Neg, AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct Defense(i64);

/// values compatible with exp
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Ord, Eq, Add, Sub, Mul, Div,
         AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct Exp(u32);

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Maxed<T> {
    pub max: T,
    pub current: T,
}

impl<T: Copy> Maxed<T> {
    fn max(init: T) -> Maxed<T> {
        Maxed {
            max: init,
            current: init,
        }
    }
}

impl<T> Maxed<T> {
    fn new(max: T, current: T) -> Maxed<T> {
        Maxed { max, current }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CharacterId(u64);

// STUB
pub struct CharacterHandler {
    rng: RngHandle,
    next_id: CharacterId,
}
