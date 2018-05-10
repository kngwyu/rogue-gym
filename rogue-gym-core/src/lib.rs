#![allow(stable_features)]
#![feature(dyn_trait, try_from, try_iterator)]
#![cfg_attr(test, feature(test, plugin))]
#![cfg_attr(test, plugin(clippy))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate enum_iterator_derive;
extern crate error_chain_mini;
#[macro_use]
extern crate error_chain_mini_derive;
extern crate fixedbitset;
extern crate num_traits;
extern crate rand;
extern crate rect_iter;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[cfg(feature = "termion")]
extern crate termion;
#[cfg(test)]
extern crate test;
extern crate tuple_map;

mod action;
mod character;
pub mod dungeon;
pub mod error;
mod fenwick;
pub mod input;
pub mod item;
mod path;
mod rng;
mod tile;
mod ui;

use character::{Player, PlayerConfig};
use dungeon::{Dungeon, DungeonStyle, Positioned, X, Y};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use error_chain_mini::ChainedError;
use input::{InputCode, Key, KeyMap};
use item::{ItemConfig, ItemHandler};
pub use tile::Tile;
use ui::{MordalKind, MordalMsg, UiState};
/// Game configuration
/// it's inteded to construct from json
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GameConfig {
    /// screen width
    pub width: i32,
    /// screen height
    pub height: i32,
    /// seed of random number generator
    /// if None, we use random value chosen by `thread_rng().gen()`
    #[serde(default)]
    pub seed: Option<u64>,
    /// dungeon configuration
    #[serde(flatten)]
    pub dungeon: DungeonStyle,
    /// item configuration
    #[serde(default)]
    pub item: ItemConfig,
    /// keymap configuration
    #[serde(default)]
    pub keymap: KeyMap,
    #[serde(default)]
    pub player: PlayerConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            width: MIN_WIDTH,
            height: MIN_HEIGHT,
            seed: None,
            dungeon: DungeonStyle::rogue(),
            item: ItemConfig::default(),
            keymap: KeyMap::default(),
            player: PlayerConfig::default(),
        }
    }
}

const MIN_WIDTH: i32 = 80;
const MAX_WIDTH: i32 = MIN_WIDTH * 2;
const MIN_HEIGHT: i32 = 24;
const MAX_HEIGHT: i32 = MIN_HEIGHT * 2;
impl GameConfig {
    fn to_global(&self) -> GameResult<GlobalConfig> {
        let seed = self.seed.unwrap_or_else(rng::gen_seed);
        let (w, h) = (self.width, self.height);
        if w < MIN_WIDTH {
            return Err(ErrorId::InvalidSetting.into_with("screen width is too narrow"));
        }
        if w > MAX_WIDTH {
            return Err(ErrorId::InvalidSetting.into_with("screen width is too wide"));
        }
        if h < MIN_HEIGHT {
            return Err(ErrorId::InvalidSetting.into_with("screen height is too narrow"));
        }
        if h > MAX_HEIGHT {
            return Err(ErrorId::InvalidSetting.into_with("screen height is too wide"));
        }
        Ok(GlobalConfig {
            width: w.into(),
            height: h.into(),
            seed,
        })
    }
    pub fn build(self) -> GameResult<RunTime> {
        let game_info = GameInfo::new();
        let config = self.to_global().chain_err("[GameConfig::build]")?;
        // TODO: invalid checking
        let mut item = ItemHandler::new(self.item.clone(), config.seed);
        // TODO: invalid checking
        let dungeon = self.dungeon
            .build(&config, &mut item, &game_info, config.seed)
            .chain_err("[GameConfig::build]")?;
        let player = self.player.build();
        Ok(RunTime {
            game_info,
            config,
            dungeon,
            item,
            ui: UiState::Dungeon,
            keymap: self.keymap,
            player,
        })
    }
}

/// API entry point of rogue core
// TODO: maybe just reference is better than Weak?
pub struct RunTime {
    game_info: GameInfo,
    config: GlobalConfig,
    dungeon: Dungeon,
    item: ItemHandler,
    player: Player,
    ui: UiState,
    pub keymap: KeyMap,
}

impl RunTime {
    fn check_interrupting(&mut self, input: input::System) -> GameResult<Vec<Reaction>> {
        use input::System::*;
        match input {
            Save => {
                let ui = UiState::Mordal(MordalKind::Quit);
                self.ui = ui.clone();
                Ok(vec![Reaction::UiTransition(ui)])
            }
            Quit => Err(ErrorId::Unimplemented.into_with(
                "[rogue_gym_core::RunTime::check_interuppting]quit command is unimplemented",
            )),
            _ => Err(ErrorId::IgnoredInput(InputCode::Sys(input))
                .into_with("[rogue_gym_core::RunTime::check_interuppting]")),
        }
    }
    pub fn draw_screen<F, E>(&self, drawer: F) -> Result<(), ChainedError<E>>
    where
        F: FnMut(Positioned<Tile>) -> Result<(), ChainedError<E>>,
        E: From<ErrorId> + ErrorKind,
    {
        // floor => item => character

        Ok(())
    }
    pub fn react_to_input(&mut self, input: InputCode) -> GameResult<Vec<Reaction>> {
        let (next_ui, res) = match self.ui {
            UiState::Dungeon => (
                None,
                match input {
                    InputCode::Sys(sys) => self.check_interrupting(sys),
                    InputCode::Act(act) | InputCode::Both { act, .. } => action::process_action(
                        act,
                        &self.config,
                        &mut self.game_info,
                        &mut self.dungeon,
                        &mut self.item,
                        &mut self.player,
                    ),
                },
            ),
            UiState::Mordal(ref mut kind) => match input {
                InputCode::Sys(sys) | InputCode::Both { sys, .. } => {
                    let res = kind.process(sys);
                    match res {
                        MordalMsg::Cancel => (
                            Some(UiState::Dungeon),
                            Ok(vec![Reaction::UiTransition(UiState::Dungeon)]),
                        ),
                        MordalMsg::Save => (
                            Some(UiState::Dungeon),
                            Err(ErrorId::Unimplemented.into_with("Save command is unimplemented")),
                        ),
                        MordalMsg::Quit => (None, Ok(vec![Reaction::Notify(GameMsg::Quit)])),
                        MordalMsg::None => (None, Ok(vec![])),
                    }
                }
                InputCode::Act(_) => (None, Err(ErrorId::IgnoredInput(input).into_err())),
            },
        };
        if let Some(next_ui) = next_ui {
            self.ui = next_ui;
        }
        res
    }
    pub fn react_to_key(&mut self, key: Key) -> GameResult<Vec<Reaction>> {
        match self.keymap.get(key) {
            Some(i) => self.react_to_input(i),
            None => {
                Err(ErrorId::InvalidInput(key).into_with("[rogue_gym_core::RunTime::react_to_key]"))
            }
        }
    }
}

/// Reaction to user input
#[derive(Clone, Debug)]
pub enum Reaction {
    /// dungeon
    Redraw,
    /// Ui State changed
    UiTransition(UiState),
    /// Game Messages,
    Notify(GameMsg),
}

#[derive(Clone, Debug)]
pub enum GameMsg {
    CantMove,
    NoDownStair,
    Quit,
}

// TODO
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    game_info: GameInfo,
    config: GlobalConfig,
}

/// Global configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub width: X,
    pub height: Y,
    pub seed: u64,
}

// only for testing
impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            width: X(80),
            height: Y(24),
            seed: 1,
        }
    }
}

/// game information shared and able to be modified by each modules
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameInfo {
    is_cleared: bool,
}

impl GameInfo {
    fn new() -> Self {
        GameInfo { is_cleared: false }
    }
}

#[cfg(test)]
mod config_test {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    #[test]
    #[ignore]
    fn print_default() {
        let game_config = GameConfig::default();
        let json = serde_json::to_string(&game_config).unwrap();
        let mut file = File::create("../data/config-default.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
    #[test]
    fn default() {
        let game_config = GameConfig::default();
        let json = serde_json::to_string(&game_config).unwrap();
        let config: GameConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, game_config);
    }
    #[test]
    fn minimum() {
        let mut file = File::open("../data/config-minimum.json").unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        let config: GameConfig = serde_json::from_str(&buf).unwrap();
        assert_eq!(config, GameConfig::default());
    }
}
