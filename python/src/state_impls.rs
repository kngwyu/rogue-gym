use rogue_gym_core::{
    error::GameResult,
    input::{Key, KeyMap},
    ui::{MordalKind, UiState},
    GameConfig, Reaction, RunTime,
};
use PlayerState;

pub(crate) struct GameStateImpl {
    pub(crate) runtime: RunTime,
    state: PlayerState,
    steps: usize,
    max_steps: usize,
}

unsafe impl Send for GameStateImpl {}

impl GameStateImpl {
    pub(crate) fn new(config: GameConfig, max_steps: usize) -> GameResult<Self> {
        let symbols = config
            .symbol_max()
            .expect("Failed to get symbol max")
            .to_byte()
            + 1;
        let mut runtime = config.build()?;
        runtime.keymap = KeyMap::ai();
        let (w, h) = runtime.screen_size();
        let mut state = PlayerState::new(w, h, symbols);
        state.update(&mut runtime)?;
        Ok(GameStateImpl {
            runtime,
            state,
            steps: 0,
            max_steps,
        })
    }
    pub(crate) fn reset(&mut self, config: GameConfig) -> GameResult<()> {
        let mut runtime = config.build()?;
        runtime.keymap = KeyMap::ai();
        self.state.update(&mut runtime)?;
        self.runtime = runtime;
        self.steps = 0;
        Ok(())
    }
    pub(crate) fn state(&self) -> PlayerState {
        self.state.clone()
    }
    pub(crate) fn symbols(&self) -> usize {
        usize::from(self.state.symbols)
    }
    pub(crate) fn react(&mut self, input: u8) -> GameResult<bool> {
        if self.steps > self.max_steps {
            return Ok(true);
        }
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
        self.steps += 1;
        Ok(dead || self.steps >= self.max_steps)
    }
}
