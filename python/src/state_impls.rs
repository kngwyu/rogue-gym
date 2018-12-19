use ndarray::{ArrayViewMut, Axis, Ix3};
use rogue_gym_core::character::player::Status;
use rogue_gym_core::{
    error::GameResult,
    input::{Key, KeyMap},
    GameConfig, Reaction, RunTime,
};
use PlayerState;

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

impl StatusFlagInner {
    pub fn len(self) -> usize {
        self.0.count_ones() as usize
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

pub(crate) struct GameStateImpl {
    pub(crate) runtime: RunTime,
    pub(crate) state: PlayerState,
}

impl GameStateImpl {
    pub(crate) fn new(config: GameConfig) -> GameResult<Self> {
        let mut runtime = config.build()?;
        runtime.keymap = KeyMap::ai();
        let (w, h) = runtime.screen_size();
        let mut state = PlayerState::new(w, h);
        state.update(&mut runtime)?;
        Ok(GameStateImpl { runtime, state })
    }
    pub(crate) fn reset(&mut self, config: GameConfig) -> GameResult<()> {
        let mut runtime = config.build()?;
        runtime.keymap = KeyMap::ai();
        self.state.update(&mut runtime)?;
        self.runtime = runtime;
        Ok(())
    }
    pub(crate) fn react(&mut self, input: u8) -> GameResult<PlayerState> {
        let res = self.runtime.react_to_key(Key::Char(input as char))?;
        for reaction in res {
            match reaction {
                Reaction::Redraw => {
                    self.state.draw_map(&self.runtime)?;
                }
                Reaction::StatusUpdated => {
                    self.state.status = self.runtime.player_status();
                }
                Reaction::UiTransition(_) => {
                    bail!("[rogue_gym_python::GameStateImpl] Ui transition happens")
                }
                Reaction::Notify(_) => {}
            }
        }
        Ok(self.state.clone())
    }
}
