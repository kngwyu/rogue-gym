use derive_more::{
    Add, AddAssign, Div, DivAssign, From, Into, Mul, MulAssign, Neg, Sub, SubAssign,
};
use enum_iterator::IntoEnumIterator;
use rect_iter::{FromTuple2, IntoTuple2};
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt};
use tuple_map::TupleMap2;

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
pub struct X(pub i32);

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
pub struct Y(pub i32);

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
    AddAssign,
    SubAssign,
    Serialize,
    Deserialize,
)]
pub struct Coord {
    pub x: X,
    pub y: Y,
}

impl Coord {
    /// create new Coord
    pub fn new<T: Into<X>, U: Into<Y>>(x: T, y: U) -> Self {
        Coord {
            x: x.into(),
            y: y.into(),
        }
    }
    pub fn is_adjacent(self, other: Coord) -> bool {
        self.euc_dist_squared(other) <= 2
    }
    /// calc dist^2 of two points
    pub fn euc_dist_squared(self, other: Coord) -> i32 {
        let (x, y) = ((self.x - other.x).0, (self.y - other.y).0);
        (x, y).map(|i| i * i).sum()
    }
    /// calc euclidian distance of two points
    pub fn euc_dist(self, other: Coord) -> f64 {
        f64::from(self.euc_dist_squared(other)).sqrt()
    }
    /// calc time step needto move from self to other,
    /// with the situation where both of two coordinates are in a same room.
    pub fn move_dist(self, other: Coord) -> i32 {
        ((self.x - other.x).0, (self.y - other.y).0)
            .map(|i| i.abs())
            .tmax()
    }
    #[inline]
    pub fn scale<T: Into<i32>>(mut self, x: T, y: T) -> Self {
        self.x *= x.into();
        self.y *= y.into();
        self
    }
    #[inline]
    pub fn slide_x<T: Into<X>>(mut self, x: T) -> Self {
        self.x += x.into();
        self
    }
    #[inline]
    pub fn slide_y<T: Into<Y>>(mut self, y: T) -> Self {
        self.y += y.into();
        self
    }
    pub fn direc_iter<F>(self, dir: Direction, predicate: F) -> DirectionIter<F>
    where
        F: FnMut(Coord) -> bool,
    {
        DirectionIter {
            cur: self,
            dir,
            predicate,
        }
    }
    pub fn is_upper(self, other: Coord) -> bool {
        self.y < other.y
    }
    pub fn is_lefter(self, other: Coord) -> bool {
        self.x < other.x
    }
    #[cfg(unix)]
    pub fn into_cursor(self) -> termion::cursor::Goto {
        let (x, y) = (self.x.0, self.y.0).map(|i| i as u16).add((1, 1));
        termion::cursor::Goto(x, y)
    }
}

impl FromTuple2<i32> for Coord {
    fn from_tuple2(t: (i32, i32)) -> Coord {
        Coord::new(t.0, t.1)
    }
}

impl IntoTuple2<i32> for Coord {
    fn into_tuple2(self) -> (i32, i32) {
        (self.x.0, self.y.0)
    }
}

impl Into<(i32, i32)> for Coord {
    fn into(self) -> (i32, i32) {
        (self.x.0, self.y.0)
    }
}

impl From<(i32, i32)> for Coord {
    fn from(t: (i32, i32)) -> Coord {
        Coord::new(t.0, t.1)
    }
}

impl Into<(usize, usize)> for Coord {
    fn into(self) -> (usize, usize) {
        (
            self.x.0.try_into().expect("convert i32 to usize"),
            self.y.0.try_into().expect("convert i32 to usize"),
        )
    }
}

pub struct Positioned<T>(pub Coord, pub T);

#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Serialize,
    Deserialize,
    IntoEnumIterator,
)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    LeftUp,
    RightUp,
    LeftDown,
    RightDown,
    Stay,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Direction::*;
        let s = match self {
            Up => "up",
            Down => "down",
            Left => "left",
            Right => "right",
            LeftUp => "left up",
            RightUp => "right up",
            LeftDown => "left down",
            RightDown => "right down",
            Stay => "stay",
        };
        write!(f, "{}", s)
    }
}

impl Direction {
    pub fn to_cd(self) -> Coord {
        use self::Direction::*;
        match self {
            Up => Coord::new(0, -1),
            Down => Coord::new(0, 1),
            Left => Coord::new(-1, 0),
            Right => Coord::new(1, 0),
            LeftUp => Coord::new(-1, -1),
            RightUp => Coord::new(1, -1),
            LeftDown => Coord::new(-1, 1),
            RightDown => Coord::new(1, 1),
            Stay => Coord::new(0, 0),
        }
    }
    pub fn x(self) -> Coord {
        use self::Direction::*;
        match self {
            Up => Coord::new(0, 0),
            Down => Coord::new(0, 0),
            Left => Coord::new(-1, 0),
            Right => Coord::new(1, 0),
            LeftUp => Coord::new(-1, 0),
            RightUp => Coord::new(1, 0),
            LeftDown => Coord::new(-1, 0),
            RightDown => Coord::new(1, 0),
            Stay => Coord::new(0, 0),
        }
    }
    pub fn y(self) -> Coord {
        use self::Direction::*;
        match self {
            Up => Coord::new(0, -1),
            Down => Coord::new(0, 1),
            Left => Coord::new(0, 0),
            Right => Coord::new(0, 0),
            LeftUp => Coord::new(0, -1),
            RightUp => Coord::new(0, -1),
            LeftDown => Coord::new(0, 1),
            RightDown => Coord::new(0, 1),
            Stay => Coord::new(0, 0),
        }
    }
    pub fn reverse(self) -> Direction {
        use self::Direction::*;
        match self {
            Up => Down,
            Left => Right,
            Right => Left,
            Down => Up,
            LeftUp => RightDown,
            RightUp => LeftDown,
            LeftDown => RightUp,
            RightDown => LeftUp,
            Stay => Stay,
        }
    }
    pub fn is_diag(self) -> bool {
        use self::Direction::*;
        match self {
            LeftUp | LeftDown | RightDown | RightUp => true,
            _ => false,
        }
    }
}

pub struct DirectionIter<F> {
    cur: Coord,
    dir: Direction,
    predicate: F,
}

impl<F> Iterator for DirectionIter<F>
where
    F: FnMut(Coord) -> bool,
{
    type Item = Coord;
    fn next(&mut self) -> Option<Coord> {
        let f = &mut self.predicate;
        if !f(self.cur) {
            return None;
        }
        let cur = self.cur;
        self.cur += self.dir.to_cd();
        Some(cur)
    }
}
