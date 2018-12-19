use fearures::MessageFlagInner;
use rogue_gym_core::{
    error::GameResult,
    input::{Key, KeyMap},
    ui::{MordalKind, UiState},
    GameConfig, Reaction, RunTime,
};
use PlayerState;

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
    pub(crate) fn react(&mut self, input: u8) -> GameResult<bool> {
        let res = self.runtime.react_to_key(Key::Char(input as char))?;
        self.state.message.reset();
        let mut dead = false;
        for reaction in res {
            match reaction {
                Reaction::Redraw => {
                    self.state.draw_map(&self.runtime)?;
                }
                Reaction::StatusUpdated => {
                    self.state.status = self.runtime.player_status();
                }
                Reaction::UiTransition(ui) => match ui {
                    UiState::Mordal(MordalKind::Grave(_)) => dead = true,
                    _ => bail!(
                        "[rogue_gym_python::GameStateImpl] Invalid ui transition {:?}",
                        ui
                    ),
                },
                Reaction::Notify(msg) => self.state.message.append(&msg),
            }
        }
        Ok(dead)
    }
}
