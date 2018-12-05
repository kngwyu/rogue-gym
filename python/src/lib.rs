#![feature(specialization)]
extern crate ndarray;
extern crate numpy;
extern crate pyo3;
extern crate rect_iter;
extern crate rogue_gym_core;

use ndarray::{Array2, ArrayViewMut, Axis, Ix3, Zip};
use numpy::PyArray3;
use pyo3::{
    basic::{PyObjectProtocol, PyObjectReprProtocol, PyObjectStrProtocol},
    exceptions::{RuntimeError, TypeError},
    prelude::*,
};
use rect_iter::{Get2D, GetMut2D, RectRange};
use rogue_gym_core::character::player::Status;
use rogue_gym_core::dungeon::{Positioned, X, Y};
use rogue_gym_core::error::*;
use rogue_gym_core::symbol;
use rogue_gym_core::{
    input::{Key, KeyMap},
    GameConfig, Reaction, RunTime,
};
use std::collections::HashMap;
use std::str::from_utf8_unchecked;

#[derive(Copy, Clone, Debug)]
struct StatusFlagInner(u32);

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
    fn len(self) -> usize {
        self.0.count_ones() as usize
    }
}

impl StatusFlagInner {
    fn copy_status(
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

/// A memory efficient representation of State.
#[pyclass]
#[derive(Clone, Debug)]
struct PlayerState {
    map: Vec<Vec<u8>>,
    history: Array2<bool>,
    status: Status,
}

impl PlayerState {
    fn new(w: X, h: Y) -> Self {
        let (w, h) = (w.0 as usize, h.0 as usize);
        PlayerState {
            map: vec![vec![b' '; w]; h],
            history: Array2::from_elem([h, w], false),
            status: Status::default(),
        }
    }
    fn update(&mut self, runtime: &RunTime) -> GameResult<()> {
        self.status = runtime.player_status();
        self.draw_map(runtime)?;
        Ok(())
    }
    fn draw_map(&mut self, runtime: &RunTime) -> GameResult<()> {
        self.history = runtime.history(&self.status).unwrap();
        runtime.draw_screen(|Positioned(cd, tile)| -> GameResult<()> {
            *self
                .map
                .try_get_mut_p(cd)
                .into_chained(|| "in python::GameState::react")? = tile.to_byte();
            Ok(())
        })
    }
    fn dungeon_str(&self) -> impl Iterator<Item = &str> {
        self.map.iter().map(|v| unsafe { from_utf8_unchecked(v) })
    }
    fn gray_image_with_offset<'py>(
        &self,
        py: Python<'py>,
        dungeon_symobols: u8,
        offset: usize,
    ) -> PyResult<&'py PyArray3<f32>> {
        let (h, w) = (self.map.len(), self.map[0].len());
        let py_array = PyArray3::zeros(py, [1 + offset, h, w], false);
        RectRange::zero_start(w, h)
            .unwrap()
            .into_iter()
            .for_each(|(x, y)| unsafe {
                let symbol = symbol::tile_to_sym(*self.map.get_xy(x, y)).unwrap();
                *py_array.uget_mut([0, y, x]) = f32::from(symbol) / f32::from(dungeon_symobols);
            });
        Ok(py_array)
    }
    fn symbol_image_with_offset<'py>(
        &self,
        py: Python<'py>,
        dungeon_symobols: u8,
        offset: usize,
    ) -> PyResult<&'py PyArray3<f32>> {
        let (h, w) = (self.map.len(), self.map[0].len());
        let channels = usize::from(dungeon_symobols);
        let py_array = PyArray3::zeros(py, [channels + offset, h, w], false);
        symbol::construct_symbol_map(&self.map, h, w, dungeon_symobols - 1, |idx| unsafe {
            py_array.uget_mut(idx)
        })
        .map_err(|e| PyErr::new::<RuntimeError, _>(format!("{}", e)))?;
        Ok(py_array)
    }
    fn copy_hist(&self, py_array: &PyArray3<f32>, offset: usize) {
        let mut array = py_array.as_array_mut();
        let hist_array = array.index_axis_mut(Axis(0), usize::from(offset));
        Zip::from(hist_array).and(&self.history).apply(|p, &r| {
            *p = if r { 1.0 } else { 0.0 };
        });
    }
}

impl<'p> PyObjectReprProtocol<'p> for PlayerState {
    type Success = String;
    type Result = PyResult<String>;
}

impl<'p> PyObjectStrProtocol<'p> for PlayerState {
    type Success = String;
    type Result = PyResult<String>;
}

impl<'p> PyObjectProtocol<'p> for PlayerState {
    fn __repr__(&'p self) -> <Self as PyObjectReprProtocol>::Result {
        let mut dungeon = self.dungeon_str().fold(String::new(), |mut res, s| {
            res.push_str(s);
            res.push('\n');
            res
        });
        dungeon.push_str(&format!("{}", self.status));
        Ok(dungeon)
    }
    fn __str__(&'p self) -> <Self as PyObjectStrProtocol>::Result {
        self.__repr__()
    }
}

#[pymethods]
impl PlayerState {
    #[getter]
    fn status(&self) -> PyResult<HashMap<String, u32>> {
        Ok(self
            .status
            .to_dict_vec()
            .into_iter()
            .map(|(s, v)| (s.to_owned(), v))
            .collect())
    }
    #[getter]
    fn dungeon(&self) -> PyResult<Vec<String>> {
        Ok(self.dungeon_str().map(|s| s.to_string()).collect())
    }
    #[getter]
    fn dungeon_level(&self) -> PyResult<u32> {
        Ok(self.status.dungeon_level)
    }
    #[getter]
    fn gold(&self) -> PyResult<u32> {
        Ok(self.status.gold)
    }
}

#[pyclass]
struct GameState {
    runtime: RunTime,
    state: PlayerState,
    config: GameConfig,
    prev_actions: Vec<Reaction>,
    dungeon_symbols: u8,
    token: PyToken,
}

#[pymethods]
impl GameState {
    #[new]
    fn __new__(obj: &PyRawObject, seed: Option<u64>, config_str: Option<String>) -> PyResult<()> {
        let mut config = if let Some(cfg) = config_str {
            GameConfig::from_json(&cfg).map_err(|e| {
                PyErr::new::<RuntimeError, _>(format!("failed to parse config, {}", e))
            })?
        } else {
            GameConfig::default()
        };
        if let Some(seed) = seed {
            config.seed = Some(u128::from(seed));
        }
        let mut runtime = config.clone().build().unwrap();
        let (w, h) = runtime.screen_size();
        runtime.keymap = KeyMap::ai();
        let symbols = config
            .symbol_max()
            .expect("Failed to get symbol max")
            .to_byte()
            + 1;
        let mut state = PlayerState::new(w, h);
        state.update(&mut runtime).unwrap();
        obj.init(|token| GameState {
            runtime,
            state,
            config,
            prev_actions: vec![Reaction::Redraw],
            dungeon_symbols: symbols,
            token,
        })
    }
    fn dungeon_channels(&self) -> usize {
        usize::from(self.dungeon_symbols)
    }
    fn screen_size(&self) -> (i32, i32) {
        (self.config.height, self.config.width)
    }
    fn set_seed(&mut self, seed: u64) -> PyResult<()> {
        self.config.seed = Some(seed as u128);
        Ok(())
    }
    /// Reset the game state
    fn reset(&mut self) -> PyResult<()> {
        let mut runtime = self.config.clone().build().unwrap();
        runtime.keymap = KeyMap::ai();
        self.state.update(&mut runtime).unwrap();
        self.runtime = runtime;
        Ok(())
    }
    /// Returns the latest game state
    fn prev(&self) -> PlayerState {
        self.state.clone()
    }
    fn react(&mut self, input: u8) -> PyResult<PlayerState> {
        let res = self
            .runtime
            .react_to_key(Key::Char(input as char))
            .map_err(|e| PyErr::new::<TypeError, _>(format!("error in rogue_gym_core: {}", e)))?;
        res.iter().for_each(|reaction| match reaction {
            Reaction::Redraw => {
                self.state.draw_map(&self.runtime).unwrap();
            }
            Reaction::StatusUpdated => {
                self.state.status = self.runtime.player_status();
            }
            // ignore ui transition
            Reaction::UiTransition(_) => {}
            Reaction::Notify(_) => {}
        });
        self.prev_actions = res;
        Ok(self.state.clone())
    }
    fn gray_image(&self, state: &PlayerState, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (self.token.py(), StatusFlagInner(flag.unwrap_or(0)));
        let array = state.gray_image_with_offset(py, self.dungeon_symbols, flag.len())?;
        flag.copy_status(&state.status, 1, &mut array.as_array_mut());
        Ok(array)
    }
    fn gray_image_with_hist(
        &self,
        state: &PlayerState,
        flag: Option<u32>,
    ) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (self.token.py(), StatusFlagInner(flag.unwrap_or(0)));
        let array = state.gray_image_with_offset(py, self.dungeon_symbols, flag.len() + 1)?;
        let offset = flag.copy_status(&state.status, 1, &mut array.as_array_mut());
        state.copy_hist(&array, offset);
        Ok(array)
    }
    /// Convert PlayerState to 3D symbol image(like AlphaGo's inputs)
    fn symbol_image(&self, state: &PlayerState, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (self.token.py(), StatusFlagInner(flag.unwrap_or(0)));
        let array = state.symbol_image_with_offset(py, self.dungeon_symbols, flag.len())?;
        flag.copy_status(
            &state.status,
            self.dungeon_channels(),
            &mut array.as_array_mut(),
        );
        Ok(array)
    }
    /// Convert PlayerState to 3D symbol image, with player history
    fn symbol_image_with_hist(
        &self,
        state: &PlayerState,
        flag: Option<u32>,
    ) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (self.token.py(), StatusFlagInner(flag.unwrap_or(0)));
        let array = state.symbol_image_with_offset(py, self.dungeon_symbols, flag.len() + 1)?;
        let offset = flag.copy_status(
            &state.status,
            self.dungeon_channels(),
            &mut array.as_array_mut(),
        );
        state.copy_hist(&array, offset);
        Ok(array)
    }
    /// Returns action history as Json
    fn dump_history(&self) -> PyResult<String> {
        self.runtime.saved_inputs_as_json().map_err(|e| {
            PyErr::new::<RuntimeError, _>(format!("error when getting history: {}", e))
        })
    }
    /// Returns config as Json
    fn dump_config(&self) -> PyResult<String> {
        self.config
            .to_json()
            .map_err(|e| PyErr::new::<RuntimeError, _>(format!("error when getting config: {}", e)))
    }
}

#[pymodinit(_rogue_gym)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GameState>()?;
    m.add_class::<PlayerState>()?;
    Ok(())
}
