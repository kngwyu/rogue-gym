#![feature(const_fn, crate_visibility_modifier, test)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate enum_iterator;
#[macro_use]
extern crate failure;
extern crate fixedbitset;
extern crate num_traits;
#[macro_use]
extern crate log;
extern crate rand;
extern crate rect_iter;
extern crate regex;
#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate smallvec;
#[cfg(feature = "termion")]
extern crate termion;
#[cfg(feature = "bench")]
extern crate test;
extern crate tuple_map;

mod actions;
pub mod character;
pub mod dungeon;
pub mod error;
mod fenwick;
pub mod input;
pub mod item;
mod rng;
pub mod tile;
pub mod ui;

use character::{player, Player};
use dungeon::{Direction, Dungeon, DungeonStyle, Positioned, X, Y};
use error::*;
use input::{InputCode, Key, KeyMap};
use item::{ItemHandler, ItemKind};
use tile::{Drawable, Tile};
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
    pub seed: Option<u128>,
    /// dungeon configuration
    #[serde(flatten)]
    pub dungeon: DungeonStyle,
    /// item configuration
    #[serde(default)]
    pub item: item::Config,
    /// keymap configuration
    #[serde(default)]
    pub keymap: KeyMap,
    /// player configuration
    #[serde(default)]
    pub player: player::Config,
    /// hide dungeon or not
    /// this setting is only for debug and don't use it when you play game
    #[serde(default = "default_hide_dungeon")]
    pub hide_dungeon: bool,
}

const fn default_hide_dungeon() -> bool {
    true
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            width: MIN_WIDTH,
            height: MIN_HEIGHT,
            seed: None,
            dungeon: DungeonStyle::rogue(),
            item: item::Config::default(),
            keymap: KeyMap::default(),
            player: player::Config::default(),
            hide_dungeon: true,
        }
    }
}

const MIN_WIDTH: i32 = 80;
const MAX_WIDTH: i32 = MIN_WIDTH * 2;
const MIN_HEIGHT: i32 = 24;
const MAX_HEIGHT: i32 = MIN_HEIGHT * 2;

impl GameConfig {
    /// construct Game configuration from json string
    pub fn from_json(json: &str) -> GameResult<Self> {
        serde_json::from_str(json).into_chained(|| "GameConfig::from_json")
    }
    fn to_global(&self) -> GameResult<GlobalConfig> {
        let seed = self.seed.unwrap_or_else(rng::gen_seed);
        let (w, h) = (self.width, self.height);
        if w < MIN_WIDTH {
            return Err(ErrorId::InvalidSetting.into_with(|| "screen width is too narrow"));
        }
        if w > MAX_WIDTH {
            return Err(ErrorId::InvalidSetting.into_with(|| "screen width is too wide"));
        }
        if h < MIN_HEIGHT {
            return Err(ErrorId::InvalidSetting.into_with(|| "screen height is too narrow"));
        }
        if h > MAX_HEIGHT {
            return Err(ErrorId::InvalidSetting.into_with(|| "screen height is too wide"));
        }
        Ok(GlobalConfig {
            width: w.into(),
            height: h.into(),
            seed,
            hide_dungeon: self.hide_dungeon,
        })
    }
    /// get runtime from config
    pub fn build(self) -> GameResult<RunTime> {
        const ERR_STR: &str = "GameConfig::build";
        let game_info = GameInfo::new();
        let config = self.to_global().chain_err(|| ERR_STR)?;
        // TODO: invalid checking
        let mut item = ItemHandler::new(self.item.clone(), config.seed);
        let mut dungeon = self
            .dungeon
            .build(&config, &mut item, &game_info, config.seed)
            .chain_err(|| ERR_STR)?;
        // TODO: invalid checking
        let mut player = self.player.build();
        item.init_player_items(&mut player.itembox, &player.config.init_items)
            .chain_err(|| ERR_STR)?;
        actions::new_level(&game_info, &mut dungeon, &mut item, &mut player, true)
            .chain_err(|| ERR_STR)?;
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
            Quit => {
                let ui = UiState::Mordal(MordalKind::Quit);
                self.ui = ui.clone();
                Ok(vec![Reaction::UiTransition(ui)])
            }
            Save => Err(ErrorId::Unimplemented.into_with(|| {
                "[rogue_gym_core::RunTime::check_interuppting] save command is unimplemented"
            })),
            _ => Err(ErrorId::IgnoredInput(InputCode::Sys(input))
                .into_with(|| "rogue_gym_core::RunTime::check_interuppting")),
        }
    }
    /// take draw function F and draw screen with it
    pub fn draw_screen<F>(&self, mut drawer: F) -> GameResult<()>
    where
        F: FnMut(Positioned<Tile>) -> GameResult<()>,
    {
        // floor => item & character
        self.dungeon.draw(&mut drawer)?;
        self.dungeon.draw_ranges().try_for_each(|path| {
            let cd = self.dungeon.path_to_cd(&path);
            if self.player.pos == path {
                return drawer(Positioned(cd, self.player.tile()));
            };
            Ok(())
        })
    }
    pub fn react_to_input(&mut self, input: InputCode) -> GameResult<Vec<Reaction>> {
        trace!("[react_to_input] input: {:?} ui: {:?}", input, self.ui);
        let (next_ui, res) = match self.ui {
            UiState::Dungeon => (
                None,
                match input {
                    InputCode::Sys(sys) => self.check_interrupting(sys),
                    InputCode::Act(act) | InputCode::Both { act, .. } => actions::process_action(
                        act,
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
                            Err(ErrorId::Unimplemented
                                .into_with(|| "Save command is unimplemented")),
                        ),
                        MordalMsg::Quit => (None, Ok(vec![Reaction::Notify(GameMsg::Quit)])),
                        MordalMsg::None => (None, Ok(vec![])),
                    }
                }
                InputCode::Act(_) => (None, Err(ErrorId::IgnoredInput(input).into())),
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
                Err(ErrorId::InvalidInput(key)
                    .into_with(|| "rogue_gym_core::RunTime::react_to_key"))
            }
        }
    }
    pub fn screen_size(&self) -> (X, Y) {
        (self.config.width, self.config.height)
    }
    pub fn player_status(&self) -> player::Status {
        let mut status = player::Status::default();
        self.player.fill_status(&mut status);
        status.gold = self
            .player
            .itembox
            .tokens()
            .find(|token| token.get().kind == ItemKind::Gold)
            .map_or(0, |token| token.get().how_many.0);
        status.dungeon_level = self.dungeon.level();
        status
    }
}

/// Reaction to user input
#[derive(Clone, Debug)]
pub enum Reaction {
    /// dungeon
    Redraw,
    /// status changed
    StatusUpdated,
    /// Ui State changed
    UiTransition(UiState),
    /// Game Messages,
    Notify(GameMsg),
}

#[derive(Clone, Debug)]
pub enum GameMsg {
    CantMove(Direction),
    // TODO: show not only kind,
    CantGetItem(ItemKind),
    GotItem { kind: ItemKind, num: u32 },
    NoDownStair,
    SecretDoor,
    Quit,
}

/// Global configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub width: X,
    pub height: Y,
    pub seed: u128,
    pub hide_dungeon: bool,
}

// only for testing
impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            width: X(80),
            height: Y(24),
            seed: 1,
            hide_dungeon: true,
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
