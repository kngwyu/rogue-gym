mod player;

/// values compatible with Hit Point
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Ord, Eq, Add, Sub, Mul, Div,
         Neg, AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct HitPoint(i64);

/// values compatible with strength
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Ord, Eq, Add, Sub, Mul, Div,
         Neg, AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct Stength(i64)

pub struct Maxed<T> {
    max: T,
    current: T,
}
