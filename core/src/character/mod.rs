pub mod enemies;
mod fight;
pub mod player;
pub use self::player::{Action, Hunger, Leveling, Player};
pub use enemies::{Enemy, EnemyHandler};
use num_traits::PrimInt;
use rand::distributions::uniform::SampleUniform;
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
pub struct HitPoint(pub i64);

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
pub struct Level(pub i64);

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
pub struct Strength(pub i64);

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
pub struct Defense(pub i64);

impl Defense {
    fn max() -> Self {
        Defense(10)
    }
}

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
pub struct Exp(pub u32);

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

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Dice<T> {
    times: usize,
    max: T,
}

impl<T> Dice<T> {
    pub const fn new(n: usize, m: T) -> Dice<T> {
        Dice { times: n, max: m }
    }
}

impl<T: Clone> Dice<T> {
    pub fn exec<I>(&self, rng: &mut RngHandle) -> T
    where
        T: Into<I>,
        I: PrimInt + SampleUniform + Into<T>,
    {
        let max: I = self.max.clone().into();
        (0..self.times)
            .fold(I::zero(), |acc, _| acc + rng.range(I::one()..=max))
            .into()
    }
}

pub trait Damage {
    fn random(self, rng: &mut RngHandle) -> HitPoint;
    fn min(self) -> HitPoint;
    fn max(self) -> HitPoint;
}

impl Damage for Dice<HitPoint> {
    fn random(self, rng: &mut RngHandle) -> HitPoint {
        (0..self.times).fold(HitPoint::default(), |acc, _| {
            acc + HitPoint::from(rng.range(1..=self.max.0))
        })
    }
    fn min(self) -> HitPoint {
        HitPoint::from(self.times as i64)
    }
    fn max(self) -> HitPoint {
        HitPoint::from(self.times as i64 * self.max.0)
    }
}

impl<I, D> Damage for I
where
    I: IntoIterator<Item = D>,
    D: ::std::ops::Deref<Target = Dice<HitPoint>>,
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
