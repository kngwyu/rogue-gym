use ndarray::{ArrayViewMut, Axis, Ix3};
use rogue_gym_core::character::player::Status;
use rogue_gym_core::GameMsg;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct MessageFlagInner(pub u32);

#[rustfmt::skip]
impl MessageFlagInner {
    const HIT_FROM: u32     = 0b000_000_001;
    const HIT_TO: u32       = 0b000_000_010;
    const MISS_TO: u32      = 0b000_000_100;
    const MISS_FROM: u32    = 0b000_001_000;
    const KILLED: u32       = 0b000_010_000;
    const SECRET_DOOR: u32  = 0b000_100_000;
    const NO_DOWNSTAIR: u32 = 0b001_000_000;
}

impl MessageFlagInner {
    pub fn new() -> Self {
        MessageFlagInner(0)
    }
    pub fn reset(&mut self) {
        self.0 = 0
    }
    pub fn append(&mut self, msg: &GameMsg) {
        let mut add = |flag: u32| self.0 |= flag;
        match msg {
            GameMsg::HitTo(_) => add(Self::HIT_TO),
            GameMsg::HitFrom(_) => add(Self::HIT_FROM),
            GameMsg::MissTo(_) => add(Self::MISS_TO),
            GameMsg::MissFrom(_) => add(Self::MISS_FROM),
            GameMsg::Killed(_) => add(Self::KILLED),
            GameMsg::SecretDoor => add(Self::SECRET_DOOR),
            GameMsg::NoDownStair => add(Self::NO_DOWNSTAIR),
            _ => (),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct StatusFlagInner(pub u32);

#[rustfmt::skip]
impl StatusFlagInner {
    const DUNGEON_LEVEL: u32 = 0b000_000_001;
    const HP_CURRENT: u32    = 0b000_000_010;
    const HP_MAX: u32        = 0b000_000_100;
    const STR_CURRENT: u32   = 0b000_001_000;
    const STR_MAX: u32       = 0b000_010_000;
    const DEFENSE: u32       = 0b000_100_000;
    const PLAYER_LEVEL: u32  = 0b001_000_000;
    const EXP: u32           = 0b010_000_000;
    const HUNGER: u32        = 0b100_000_000;
}

impl From<Option<u32>> for StatusFlagInner {
    fn from(u: Option<u32>) -> Self {
        StatusFlagInner(u.unwrap_or(0))
    }
}

impl StatusFlagInner {
    pub fn len(self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn to_vector(&self, status: &Status) -> Vec<i32> {
        let mut res = vec![];
        {
            let mut add = |flag: u32, value| {
                if (self.0 & flag) != 0 {
                    res.push(value)
                }
            };
            add(Self::DUNGEON_LEVEL, status.dungeon_level as i32);
            add(Self::HP_CURRENT, status.hp.current.0 as i32);
            add(Self::HP_MAX, status.hp.max.0 as i32);
            add(Self::STR_CURRENT, status.strength.current.0 as i32);
            add(Self::STR_MAX, status.strength.max.0 as i32);
            add(Self::DEFENSE, status.defense.0 as i32);
            add(Self::PLAYER_LEVEL, status.player_level as i32);
            add(Self::EXP, status.exp.0 as i32);
            add(Self::HUNGER, status.hunger_level.to_u32() as i32);
        }
        res
    }
    pub fn copy_status(
        self,
        status: &Status,
        start: usize,
        array: &mut ArrayViewMut<f32, Ix3>,
    ) -> usize {
        let mut offset = start;
        {
            let mut copy = |flag: u32, value| {
                if (self.0 & flag) != 0 {
                    let mut array = array.index_axis_mut(Axis(0), offset);
                    array.iter_mut().for_each(|elem| {
                        *elem = value as f32;
                    });
                    offset += 1;
                }
            };
            copy(Self::DUNGEON_LEVEL, status.dungeon_level as i32);
            copy(Self::HP_CURRENT, status.hp.current.0 as i32);
            copy(Self::HP_MAX, status.hp.max.0 as i32);
            copy(Self::STR_CURRENT, status.strength.current.0 as i32);
            copy(Self::STR_MAX, status.strength.max.0 as i32);
            copy(Self::DEFENSE, status.defense.0 as i32);
            copy(Self::PLAYER_LEVEL, status.player_level as i32);
            copy(Self::EXP, status.exp.0 as i32);
            copy(Self::HUNGER, status.hunger_level.to_u32() as i32);
        }
        offset
    }
}
