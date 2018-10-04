pub mod enemies;
pub mod player;
pub use self::player::{Action, Hunger, Leveling, Player};
use rng::RngHandle;

/// values compatible with Hit Point
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Display,
    Neg,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct HitPoint(i64);

/// values compatible with strength
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Display,
    Neg,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct Strength(i64);

/// values compatible with defense power of Armors
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Display,
    Neg,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct Defense(i64);

/// values compatible with exp
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Display,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct Exp(u32);

macro_rules! from_impls {
    ($t: ident, $($from: ty)+) => {
        $(impl From<$from> for $t {
            fn from(i: $from) -> Self {
                $t(i.into())
            }
        })+
    };
}

from_impls!(HitPoint, i8 u8 i16 u16 i32 u32);
from_impls!(Strength, i8 u8 i16 u16 i32 u32);
from_impls!(Defense, i8 u8 i16 u16 i32 u32);
from_impls!(Exp, u8 u16);

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

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Dice {
    times: i32,
    max: i32,
}

impl Dice {
    pub const fn new(n: i32, m: i32) -> Dice {
        Dice { times: n, max: m }
    }
}

pub trait Damage {
    fn random(self, rng: &mut RngHandle) -> HitPoint;
    fn min(self) -> HitPoint;
    fn max(self) -> HitPoint;
}

impl Damage for Dice {
    fn random(self, rng: &mut RngHandle) -> HitPoint {
        (0..self.times).fold(HitPoint::default(), |acc, _| {
            acc + HitPoint::from(rng.range(1..=self.max))
        })
    }
    fn min(self) -> HitPoint {
        HitPoint::from(self.times)
    }
    fn max(self) -> HitPoint {
        HitPoint::from(i64::from(self.times) * i64::from(self.max))
    }
}

impl<I, D> Damage for I
where
    I: IntoIterator<Item = D>,
    D: ::std::ops::Deref<Target = Dice>,
{
    fn random(self, rng: &mut RngHandle) -> HitPoint {
        self.into_iter()
            .fold(HitPoint::default(), |acc, d| acc + d.random(rng))
    }
    fn max(self) -> HitPoint {
        self.into_iter()
            .fold(HitPoint::default(), |acc, d| acc + d.max())
    }
    fn min(self) -> HitPoint {
        self.into_iter()
            .fold(HitPoint::default(), |acc, d| acc + d.min())
    }
}
