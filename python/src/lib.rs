#![feature(specialization)]
extern crate ndarray;
extern crate numpy;
extern crate pyo3;
extern crate rect_iter;
extern crate rogue_gym_core;

use ndarray::{Array2, ArrayViewMut, Axis, Ix2, Zip};
use numpy::PyArray3;
use pyo3::{
    basic::{PyObjectReprProtocol, PyObjectStrProtocol},
    exc,
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

/// Memory efficient representation of State.
#[pyclass]
#[derive(Clone, Debug)]
struct PlayerState {
    map: Vec<Vec<u8>>,
    history: Array2<bool>,
    channels: u8,
    status: Status,
}

impl PlayerState {
    fn new(w: X, h: Y, channels: u8) -> Self {
        let (w, h) = (w.0 as usize, h.0 as usize);
        PlayerState {
            map: vec![vec![b' '; w]; h],
            history: Array2::from_elem([h, w], false),
            channels,
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
    fn symbol_image_<'py>(&self, array: &'py PyArray3<f32>, h: usize, w: usize) -> PyResult<()> {
        symbol::construct_symbol_map(&self.map, h, w, self.channels - 1, |idx| unsafe {
            array.uget_mut(idx)
        })
        .map_err(|e| PyErr::new::<exc::RuntimeError, _>(format!("{}", e)))?;
        Ok(())
    }
    fn gray_image_with_offset<'py>(
        &self,
        py: Python<'py>,
        offset: usize,
    ) -> PyResult<&'py PyArray3<f32>> {
        let (h, w) = (self.map.len(), self.map[0].len());
        let py_array = PyArray3::zeros(py, [1 + offset, h, w], false);
        RectRange::zero_start(w, h)
            .unwrap()
            .into_iter()
            .for_each(|(x, y)| unsafe {
                let symbol = symbol::tile_to_sym(*self.map.get_xy(x, y)).unwrap();
                *py_array.uget_mut([0, y, x]) = symbol as f32 / self.channels as f32;
            });
        Ok(py_array)
    }
    fn symbol_image_with_offset<'py>(
        &self,
        py: Python<'py>,
        offset: usize,
    ) -> PyResult<&'py PyArray3<f32>> {
        let (h, w) = (self.map.len(), self.map[0].len());
        let channels = usize::from(self.channels);
        let py_array = PyArray3::zeros(py, [channels + offset, h, w], false);
        self.symbol_image_(py_array, h, w)?;
        Ok(py_array)
    }
    fn copy_hist<'py>(&self, pyarray: &mut ArrayViewMut<f32, Ix2>) {
        Zip::from(pyarray).and(&self.history).apply(|p, &r| {
            *p = if r { 1.0 } else { 0.0 };
        });
    }
    fn symbol_image_with_hist<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray3<f32>> {
        let py_array = self.symbol_image_with_offset(py, 1)?;
        let mut array = py_array.as_array_mut()?;
        let mut hist_array = array.subview_mut(Axis(0), usize::from(self.channels));
        self.copy_hist(&mut hist_array);
        Ok(py_array)
    }
}

impl ::pyo3::IntoPyObject for PlayerState {
    fn into_object(self, py: ::pyo3::Python) -> ::pyo3::PyObject {
        ::pyo3::Py::new(py, |_| self).unwrap().into_object(py)
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
    token: PyToken,
}

#[pymethods]
impl GameState {
    #[new]
    fn __new__(obj: &PyRawObject, seed: Option<u64>, config_str: Option<String>) -> PyResult<()> {
        let mut config = if let Some(cfg) = config_str {
            GameConfig::from_json(&cfg).map_err(|e| {
                PyErr::new::<exc::RuntimeError, _>(format!("failed to parse config, {}", e))
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
        let channels = config
            .symbol_max()
            .expect("Failed to get symbol max")
            .to_byte()
            + 1;
        let mut state = PlayerState::new(w, h, channels);
        state.update(&mut runtime).unwrap();
        obj.init(|token| GameState {
            runtime,
            state,
            config,
            prev_actions: vec![Reaction::Redraw],
            token,
        })
    }
    fn channels(&self) -> i32 {
        i32::from(self.state.channels)
    }
    fn screen_size(&self) -> (i32, i32) {
        (self.config.height, self.config.width)
    }
    fn feature_dims(&self) -> (i32, i32, i32) {
        (
            i32::from(self.state.channels),
            self.config.height,
            self.config.width,
        )
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
            .map_err(|e| {
                PyErr::new::<exc::TypeError, _>(format!("error in rogue_gym_core: {}", e))
            })?;
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
    fn get_gray_image(&self, state: &PlayerState) -> PyResult<&PyArray3<f32>> {
        state.gray_image_with_offset(self.token.py(), 0)
    }
    fn get_gray_image_with_hist(&self, state: &PlayerState) -> PyResult<&PyArray3<f32>> {
        let py_array = state.gray_image_with_offset(self.token.py(), 1)?;
        let mut array = py_array.as_array_mut()?;
        let mut hist_array = array.subview_mut(Axis(0), 1);
        state.copy_hist(&mut hist_array);
        Ok(py_array)
    }
    /// Convert PlayerState to 3D symbol image(like AlphaGo's inputs)
    fn get_symbol_image(&self, state: &PlayerState) -> PyResult<&PyArray3<f32>> {
        let py = self.token.py();
        state.symbol_image_with_offset(py, 0)
    }
    /// Convert PlayerState to 3D symbol image, with player history
    fn get_symbol_image_with_hist(&self, state: &PlayerState) -> PyResult<&PyArray3<f32>> {
        let py = self.token.py();
        state.symbol_image_with_hist(py)
    }
    /// Returns action history as Json
    fn dump_history(&self) -> PyResult<String> {
        self.runtime.saved_inputs_as_json().map_err(|e| {
            PyErr::new::<exc::RuntimeError, _>(format!("error when getting history: {}", e))
        })
    }
    /// Returns config as Json
    fn dump_config(&self) -> PyResult<String> {
        self.config.to_json().map_err(|e| {
            PyErr::new::<exc::RuntimeError, _>(format!("error when getting config: {}", e))
        })
    }
}

#[pymodinit(_rogue_gym)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GameState>()?;
    m.add_class::<PlayerState>()?;
    Ok(())
}
