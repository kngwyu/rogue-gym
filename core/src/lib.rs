#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate enum_iterator;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[cfg(feature = "bench")]
extern crate test;

mod actions;
use std::fs::File;
use std::io::{self, Read};
pub mod character;
pub mod dungeon;
pub mod error;
mod fenwick;
pub mod input;
pub mod item;
mod rng;
mod smallstr;
pub mod symbol;
pub mod tile;
pub mod ui;

use crate::character::{enemies, player, EnemyHandler, Player};
use crate::dungeon::{Direction, Dungeon, DungeonStyle, Positioned, X, Y};
use anyhow::{bail, Context};
use error::*;
use input::{InputCode, Key, KeyMap};
use item::{ItemHandler, ItemKind};
use log::{debug, trace};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
pub use smallstr::SmallStr;
use tile::{Drawable, Tile};
use ui::{MordalKind, MordalMsg, UiState};

/// Game configuration
/// it's inteded to construct from json
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GameConfig {
    /// screen width
    #[serde(default = "default_screen_width")]
    #[serde(skip_serializing_if = "is_default_width")]
    pub width: i32,
    /// screen height
    #[serde(default = "default_screen_height")]
    #[serde(skip_serializing_if = "is_default_height")]
    pub height: i32,
    /// seed of random number generator
    /// if None, we use random value chosen by `thread_rng().gen()`
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub seed: Option<u128>,
    /// The half-open range from which you choose the game seed
    /// Only available when seed == None
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub seed_range: Option<[u128; 2]>,
    /// dungeon configuration
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub dungeon: DungeonStyle,
    /// item configuration
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub item: item::Config,
    /// keymap configuration
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub keymap: KeyMap,
    /// player configuration
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub player: player::Config,
    /// enemy configuration
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub enemies: enemies::Config,
    /// hide dungeon or not
    /// this setting is only for debugging and don't use it when you play game
    #[serde(default = "default_hide_dungeon")]
    pub hide_dungeon: bool,
}

unsafe impl Send for GameConfig {}

fn is_default<T>(s: &T) -> bool
where
    T: Default + PartialEq,
{
    cfg!(not(test)) && *s == T::default()
}

const fn default_screen_width() -> i32 {
    DEFAULT_WIDTH
}

fn is_default_width(w: &i32) -> bool {
    cfg!(not(test)) && *w == DEFAULT_WIDTH
}

const fn default_screen_height() -> i32 {
    DEFAULT_HEIGHT
}

fn is_default_height(h: &i32) -> bool {
    cfg!(not(test)) && *h == DEFAULT_HEIGHT
}

const fn default_hide_dungeon() -> bool {
    true
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            seed: Default::default(),
            seed_range: Default::default(),
            dungeon: DungeonStyle::default(),
            item: item::Config::default(),
            keymap: KeyMap::default(),
            player: player::Config::default(),
            enemies: enemies::Config::default(),
            hide_dungeon: default_hide_dungeon(),
        }
    }
}

pub const DEFAULT_WIDTH: i32 = 80;
pub const DEFAULT_HEIGHT: i32 = 24;
pub const MAX_WIDTH: i32 = DEFAULT_WIDTH * 2;
pub const MAX_HEIGHT: i32 = DEFAULT_HEIGHT * 2;

pub const MIN_WIDTH: i32 = 32;
pub const MIN_HEIGHT: i32 = 16;

impl GameConfig {
    /// construct Game configuration from json string
    pub fn from_json(json: &str) -> GameResult<Self> {
        serde_json::from_str(json).context("GameConfig::from_json")
    }
    pub fn to_json(&self) -> GameResult<String> {
        serde_json::to_string_pretty(self).context("GameConfig::to_json")
    }
    pub fn symbol_max(&self) -> Option<symbol::Symbol> {
        match self.enemies.tile_max() {
            Some(t) => symbol::Symbol::from_tile(t.into()),
            None => symbol::Symbol::from_tile(b'A'.into()).map(|s| s.decrement()),
        }
    }
    fn to_global(&self) -> GameResult<GlobalConfig> {
        let seed = if let Some(s) = self.seed {
            s
        } else {
            if let Some(r) = self.seed_range {
                rng::gen_ranged_seed(r[0], r[1])
            } else {
                rng::gen_seed()
            }
        };
        let (w, h) = (self.width, self.height);
        if w < MIN_WIDTH {
            bail!(ErrorKind::InvalidSetting(
                "screen width is too narrow".into()
            ));
        }
        if w > MAX_WIDTH {
            bail!(ErrorKind::InvalidSetting("screen width is too wide".into()));
        }
        if h < MIN_HEIGHT {
            bail!(ErrorKind::InvalidSetting(
                "screen height is too narrow".into()
            ));
        }
        if h > MAX_HEIGHT {
            bail!(ErrorKind::InvalidSetting(
                "screen height is too wide".into()
            ));
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
        let config = self.to_global().context(ERR_STR)?;
        debug!("Building dungeon with seed {}", config.seed);
        // TODO: invalid checking
        let mut item = ItemHandler::new(self.item.clone(), config.seed);
        let mut enemies = self.enemies.build(config.seed);
        let mut dungeon = self
            .dungeon
            .build(&config, &mut item, &mut enemies, &game_info, config.seed)
            .context(ERR_STR)?;
        // TODO: invalid checking
        let mut player = self.player.build();
        player.init_items(&mut item).context(ERR_STR)?;
        actions::new_level(
            &game_info,
            &mut *dungeon,
            &mut item,
            &mut player,
            &mut enemies,
            true,
        )
        .context(ERR_STR)?;
        Ok(RunTime {
            game_info,
            config,
            dungeon,
            item,
            player,
            enemies,
            ui: UiState::Dungeon,
            saved_inputs: vec![],
            keymap: self.keymap,
        })
    }
}

/// API entry point of rogue core
pub struct RunTime {
    game_info: GameInfo,
    config: GlobalConfig,
    dungeon: Box<dyn Dungeon>,
    item: ItemHandler,
    player: Player,
    ui: UiState,
    saved_inputs: Vec<InputCode>,
    enemies: EnemyHandler,
    pub keymap: KeyMap,
}

impl RunTime {
    fn check_interrupting(&mut self, input: input::System) -> GameResult<Vec<Reaction>> {
        use input::System;
        match input {
            System::Quit => {
                let ui = UiState::Mordal(MordalKind::Quit);
                self.ui = ui.clone();
                Ok(vec![Reaction::UiTransition(ui)])
            }
            System::Inventory => {
                let ui = UiState::Mordal(MordalKind::Inventory);
                self.ui = ui.clone();
                Ok(vec![Reaction::UiTransition(ui)])
            }
            System::Save => bail!(ErrorKind::Unimplemented("Save command")),
            _ => Err(ErrorKind::IgnoredInput(InputCode::Sys(input)))
                .context("rogue_gym_core::RunTime::check_interuppting"),
        }
    }
    /// take draw function F and draw screen with it
    pub fn draw_screen(
        &self,
        mut drawer: impl FnMut(Positioned<Tile>) -> GameResult<()>,
    ) -> GameResult<()> {
        // floor => item & character
        self.dungeon.draw(&mut drawer)?;
        self.dungeon.draw_ranges().into_iter().try_for_each(|path| {
            let cd = self.dungeon.path_to_cd(&path);
            if self.player.pos == path {
                return drawer(Positioned(cd, self.player.tile()));
            };
            if let Some(item) = self.dungeon.get_item(&path) {
                return drawer(Positioned(cd, item.tile()));
            }
            if let Some(enemy) = self.enemies.get_enemy(&path) {
                if self.dungeon.draw_enemy(&self.player.pos, &path) {
                    return drawer(Positioned(cd, enemy.tile()));
                }
            }
            Ok(())
        })
    }
    pub fn react_to_input(&mut self, input: InputCode) -> GameResult<Vec<Reaction>> {
        trace!("[react_to_input] input: {:?} ui: {:?}", input, self.ui);
        self.saved_inputs.push(input);
        let (next_ui, res) = match self.ui {
            UiState::Dungeon => match input {
                InputCode::Sys(sys) => (None, self.check_interrupting(sys)?),
                InputCode::Act(act) | InputCode::Both { act, .. } => actions::process_action(
                    act,
                    &mut self.game_info,
                    &mut *self.dungeon,
                    &mut self.item,
                    &mut self.player,
                    &mut self.enemies,
                )?,
            },
            UiState::Mordal(ref mut kind) => match input {
                InputCode::Sys(sys) | InputCode::Both { sys, .. } => {
                    let res = kind.process(sys);
                    match res {
                        MordalMsg::Cancel => (
                            Some(UiState::Dungeon),
                            vec![Reaction::UiTransition(UiState::Dungeon)],
                        ),
                        MordalMsg::Save => bail!(ErrorKind::Unimplemented("Save command")),
                        MordalMsg::Quit => (None, vec![Reaction::Notify(GameMsg::Quit)]),
                        MordalMsg::None => (None, vec![]),
                    }
                }
                InputCode::Act(_) => bail!(ErrorKind::IgnoredInput(input)),
            },
        };
        if let Some(next_ui) = next_ui {
            self.ui = next_ui;
        }
        Ok(res)
    }
    pub fn react_to_key(&mut self, key: Key) -> GameResult<Vec<Reaction>> {
        match self.keymap.get(key) {
            Some(i) => self.react_to_input(i),
            None => Err(ErrorKind::InvalidInput(key).into()),
        }
    }
    pub fn is_cancel(&self, key: Key) -> GameResult<bool> {
        match self.keymap.get(key) {
            Some(i) => match i {
                InputCode::Both { sys, .. } | InputCode::Sys(sys) => match sys {
                    input::System::Cancel | input::System::Enter | input::System::Continue => {
                        Ok(true)
                    }
                    _ => Ok(false),
                },
                _ => Ok(false),
            },
            None => Err(ErrorKind::InvalidInput(key).into()),
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
    pub fn saved_inputs(&self) -> &[InputCode] {
        &self.saved_inputs
    }
    pub fn saved_inputs_as_json(&self) -> GameResult<String> {
        serde_json::to_string_pretty(&self.saved_inputs)
            .context("Runtime::saved_inputs_json: Failed to serialize")
    }
    pub fn history(&self, player_stat: &player::Status) -> Option<Array2<bool>> {
        self.dungeon.get_history(&player_stat)
    }
    pub fn itembox(&self) -> &item::ItemBox {
        debug!("itembox {:?}", self.player.itembox);
        &self.player.itembox
    }
}

pub fn json_to_inputs(json: &str) -> GameResult<Vec<InputCode>> {
    serde_json::from_str(json).context("json_to_inputs: Failed to deserialize")
}

/// Reaction to user input
#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum GameMsg {
    CantMove(Direction),
    CantGetItem(ItemKind),
    GotItem { kind: ItemKind, num: u32 },
    HitTo(SmallStr),
    HitFrom(SmallStr),
    MissTo(SmallStr),
    MissFrom(SmallStr),
    Killed(SmallStr),
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

pub fn read_file(name: &str) -> io::Result<String> {
    let mut file = File::open(name)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
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
        let config: GameConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config, GameConfig::default());
    }
}
