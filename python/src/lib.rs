#![feature(specialization)]
#[macro_use]
extern crate failure;
extern crate ndarray;
extern crate numpy;
extern crate pyo3;
extern crate rect_iter;
extern crate rogue_gym_core;

mod fearures;
mod state_impls;
mod thread_impls;

use fearures::{MessageFlagInner, StatusFlagInner};
use ndarray::{Array2, Axis, Zip};
use numpy::PyArray3;
use pyo3::{
    basic::{PyObjectProtocol, PyObjectReprProtocol, PyObjectStrProtocol},
    exceptions::RuntimeError,
    prelude::*,
};
use rect_iter::{Get2D, GetMut2D, RectRange};
use rogue_gym_core::character::player::Status;
use rogue_gym_core::dungeon::{Positioned, X, Y};
use rogue_gym_core::{error::*, symbol, GameConfig, RunTime};
use state_impls::GameStateImpl;
use std::collections::HashMap;
use std::str::from_utf8_unchecked;

fn pyresult<T>(result: GameResult<T>) -> PyResult<T> {
    pyresult_with(result, "Error in rogue-gym")
}

fn pyresult_with<T>(result: GameResult<T>, msg: &str) -> PyResult<T> {
    result.map_err(|e| PyErr::new::<RuntimeError, _>(format!("{}: {}", msg, e)))
}

/// A memory efficient representation of Agent observation.
#[pyclass]
#[derive(Clone, Debug, PartialEq)]
struct PlayerState {
    map: Vec<Vec<u8>>,
    history: Array2<bool>,
    status: Status,
    message: MessageFlagInner,
}

impl PlayerState {
    fn new(w: X, h: Y) -> Self {
        let (w, h) = (w.0 as usize, h.0 as usize);
        PlayerState {
            map: vec![vec![b' '; w]; h],
            history: Array2::from_elem([h, w], false),
            status: Status::default(),
            message: MessageFlagInner::new(),
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
    fn status_vec(&self, flag: u32) -> Vec<i32> {
        let flag = StatusFlagInner(flag);
        flag.to_vector(&self.status)
    }
}

struct StateConverter {
    token: PyToken,
    symbols: u8,
}

impl StateConverter {
    fn new(token: PyToken, symbols: u8) -> Self {
        StateConverter { token, symbols }
    }
    /// Convert PlayerState with 2D gray image dungeon
    fn gray_image(&self, state: &PlayerState, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (self.token.py(), StatusFlagInner(flag.unwrap_or(0)));
        let array = state.gray_image_with_offset(py, self.symbols, flag.len())?;
        flag.copy_status(&state.status, 1, &mut array.as_array_mut());
        Ok(array)
    }
    fn gray_image_with_hist(
        &self,
        state: &PlayerState,
        flag: Option<u32>,
    ) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (self.token.py(), StatusFlagInner(flag.unwrap_or(0)));
        let array = state.gray_image_with_offset(py, self.symbols, flag.len() + 1)?;
        let offset = flag.copy_status(&state.status, 1, &mut array.as_array_mut());
        state.copy_hist(&array, offset);
        Ok(array)
    }
    /// Convert PlayerState with 3D symbol image dungeon(like AlphaGo's inputs)
    fn symbol_image(&self, state: &PlayerState, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (self.token.py(), StatusFlagInner(flag.unwrap_or(0)));
        let array = state.symbol_image_with_offset(py, self.symbols, flag.len())?;
        flag.copy_status(
            &state.status,
            usize::from(self.symbols),
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
        let array = state.symbol_image_with_offset(py, self.symbols, flag.len() + 1)?;
        let offset = flag.copy_status(
            &state.status,
            usize::from(self.symbols),
            &mut array.as_array_mut(),
        );
        state.copy_hist(&array, offset);
        Ok(array)
    }
}

#[pyclass]
struct GameState {
    inner: GameStateImpl,
    config: GameConfig,
    state_converter: StateConverter,
}

#[pymethods]
impl GameState {
    #[new]
    fn __new__(obj: &PyRawObject, seed: Option<u64>, config_str: Option<String>) -> PyResult<()> {
        let mut config = if let Some(cfg) = config_str {
            pyresult_with(GameConfig::from_json(&cfg), "Failed to parse config")?
        } else {
            GameConfig::default()
        };
        if let Some(seed) = seed {
            config.seed = Some(u128::from(seed));
        }
        let symbols = config
            .symbol_max()
            .expect("Failed to get symbol max")
            .to_byte()
            + 1;
        let inner = pyresult(GameStateImpl::new(config.clone()))?;
        obj.init(|token| GameState {
            inner,
            config,
            state_converter: StateConverter::new(token, symbols),
        })
    }
    fn dungeon_channels(&self) -> usize {
        usize::from(self.state_converter.symbols)
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
        pyresult(self.inner.reset(self.config.clone()))
    }
    /// Returns the latest game state
    fn prev(&self) -> PlayerState {
        self.inner.state()
    }
    fn react(&mut self, input: u8) -> PyResult<bool> {
        pyresult(self.inner.react(input))
    }
    /// Convert PlayerState with 2D gray image dungeon
    fn gray_image(&self, state: &PlayerState, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        self.state_converter.gray_image(state, flag)
    }
    /// Convert PlayerState with 2D gray image dungeon + history
    fn gray_image_with_hist(
        &self,
        state: &PlayerState,
        flag: Option<u32>,
    ) -> PyResult<&PyArray3<f32>> {
        self.state_converter.gray_image_with_hist(state, flag)
    }
    /// Convert PlayerState with 3D symbol image dungeon(like AlphaGo's inputs)
    fn symbol_image(&self, state: &PlayerState, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        self.state_converter.symbol_image(state, flag)
    }
    /// Convert PlayerState with 3D symbol image dungeon + history
    fn symbol_image_with_hist(
        &self,
        state: &PlayerState,
        flag: Option<u32>,
    ) -> PyResult<&PyArray3<f32>> {
        self.state_converter.symbol_image_with_hist(state, flag)
    }
    /// Returns action history as Json
    fn dump_history(&self) -> PyResult<String> {
        pyresult_with(
            self.inner.runtime.saved_inputs_as_json(),
            "Error when getting history",
        )
    }
    /// Returns config as Json
    fn dump_config(&self) -> PyResult<String> {
        pyresult_with(self.config.to_json(), "Error when getting config")
    }
}

#[pymodinit(_rogue_gym)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GameState>()?;
    m.add_class::<PlayerState>()?;
    Ok(())
}
