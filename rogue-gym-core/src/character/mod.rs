pub use self::player::{Action, Hunger, Player, PlayerStatus};

mod player;
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
