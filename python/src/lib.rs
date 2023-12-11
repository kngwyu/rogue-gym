mod flags;
mod state_impls;
mod thread_impls;

use anyhow::Context;
use flags::{MessageFlagInner, StatusFlagInner};
use ndarray::{Array2, Axis, Zip};
use numpy::PyArray3;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use rect_iter::{Get2D, GetMut2D, RectRange};
use rogue_gym_core::character::player::Status;
use rogue_gym_core::dungeon::{Positioned, X, Y};
use rogue_gym_core::{error::*, symbol, GameConfig, RunTime};
use state_impls::GameStateImpl;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::from_utf8_unchecked;
use thread_impls::ThreadConductor;

fn pyresult<T, E: Display>(result: Result<T, E>) -> PyResult<T> {
    pyresult_with(result, "Error in rogue-gym")
}

fn pyresult_with<T, E: Display>(result: Result<T, E>, msg: &str) -> PyResult<T> {
    result.map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("{}: {}", msg, e)))
}

/// A memory efficient representation of Agent observation.
#[pyclass]
#[derive(Clone, Debug, PartialEq)]
struct PlayerState {
    map: Vec<Vec<u8>>,
    history: Array2<bool>,
    status: Status,
    symbols: u8,
    message: MessageFlagInner,
    is_terminal: bool,
}

impl PlayerState {
    fn new(w: X, h: Y, symbols: u8) -> Self {
        let (w, h) = (w.0 as usize, h.0 as usize);
        PlayerState {
            map: vec![vec![b' '; w]; h],
            history: Array2::from_elem([h, w], false),
            status: Status::default(),
            symbols,
            message: MessageFlagInner::new(),
            is_terminal: false,
        }
    }
    fn reset(&mut self, runtime: &RunTime) -> GameResult<()> {
        self.status = runtime.player_status();
        self.draw_map(runtime)?;
        self.message = MessageFlagInner::new();
        self.is_terminal = false;
        Ok(())
    }
    fn draw_map(&mut self, runtime: &RunTime) -> GameResult<()> {
        self.history = runtime.history(&self.status).unwrap();
        runtime.draw_screen(|Positioned(cd, tile)| -> GameResult<()> {
            *self
                .map
                .try_get_mut_p(cd)
                .context("in python::GameState::react")? = tile.to_byte();
            Ok(())
        })
    }
    fn dungeon_str(&self) -> impl Iterator<Item = &str> {
        self.map.iter().map(|v| unsafe { from_utf8_unchecked(v) })
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
                *py_array.uget_mut([0, y, x]) = f32::from(symbol) / f32::from(self.symbols);
            });
        Ok(py_array)
    }
    fn symbol_image_with_offset<'py>(
        &self,
        py: Python<'py>,
        offset: usize,
    ) -> PyResult<&'py PyArray3<f32>> {
        let (h, w) = (self.map.len(), self.map[0].len());
        let channels = usize::from(self.symbols);
        let py_array = PyArray3::zeros(py, [channels + offset, h, w], false);
        pyresult(symbol::construct_symbol_map(
            &self.map,
            h,
            w,
            self.symbols - 1,
            |idx| unsafe { py_array.uget_mut(idx) },
        ))?;
        Ok(py_array)
    }
    fn copy_hist(&self, py_array: &PyArray3<f32>, offset: usize) {
        let mut array = unsafe { py_array.as_array_mut() };
        let hist_array = array.index_axis_mut(Axis(0), usize::from(offset));
        Zip::from(hist_array).and(&self.history).for_each(|p, &r| {
            *p = if r { 1.0 } else { 0.0 };
        });
    }
}

#[pymethods]
impl PlayerState {
    fn __repr__(&self) -> String {
        let mut dungeon = self.dungeon_str().fold(String::new(), |mut res, s| {
            res.push_str(s);
            res.push('\n');
            res
        });
        dungeon.push_str(&format!("{}", self.status));
        dungeon
    }
    fn __str__(&self) -> String {
        self.__repr__()
    }

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
    #[getter]
    fn symbols(&self) -> PyResult<usize> {
        Ok(usize::from(self.symbols))
    }
    #[getter]
    fn is_terminal(&self) -> PyResult<bool> {
        Ok(self.is_terminal)
    }
    fn status_vec(&self, flag: u32) -> Vec<i32> {
        let flag = StatusFlagInner(flag);
        flag.to_vector(&self.status)
    }
    fn gray_image(&self, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (
            unsafe { Python::assume_gil_acquired() },
            StatusFlagInner::from(flag),
        );
        let array = self.gray_image_with_offset(py, flag.len())?;
        flag.copy_status(&self.status, 1, &mut unsafe { array.as_array_mut() });
        Ok(array)
    }
    fn gray_image_with_hist(&self, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (
            unsafe { Python::assume_gil_acquired() },
            StatusFlagInner::from(flag),
        );
        let array = self.gray_image_with_offset(py, flag.len() + 1)?;
        let offset = flag.copy_status(&self.status, 1, &mut unsafe { array.as_array_mut() });
        self.copy_hist(&array, offset);
        Ok(array)
    }
    /// Convert PlayerSelf with 3D symbol image dungeon(like AlphaGo's inputs)
    fn symbol_image(&self, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (
            unsafe { Python::assume_gil_acquired() },
            StatusFlagInner::from(flag),
        );
        let array = self.symbol_image_with_offset(py, flag.len())?;
        flag.copy_status(&self.status, usize::from(self.symbols), &mut unsafe {
            array.as_array_mut()
        });
        Ok(array)
    }
    /// Convert PlayerState to 3D symbol image, with player history
    fn symbol_image_with_hist(&self, flag: Option<u32>) -> PyResult<&PyArray3<f32>> {
        let (py, flag) = (
            unsafe { Python::assume_gil_acquired() },
            StatusFlagInner::from(flag),
        );
        let array = self.symbol_image_with_offset(py, flag.len() + 1)?;
        let offset = flag.copy_status(&self.status, usize::from(self.symbols), &mut unsafe {
            array.as_array_mut()
        });
        self.copy_hist(&array, offset);
        Ok(array)
    }
}

#[pyclass]
struct GameState {
    inner: GameStateImpl,
    config: GameConfig,
}

#[pymethods]
impl GameState {
    #[new]
    fn __new__(max_steps: usize, config_str: Option<String>) -> PyResult<Self> {
        let config = if let Some(cfg) = config_str {
            pyresult_with(GameConfig::from_json(&cfg), "Failed to parse config")?
        } else {
            GameConfig::default()
        };
        let inner = pyresult(GameStateImpl::new(config.clone(), max_steps))?;
        Ok(GameState { inner, config })
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
    fn react(&mut self, input: u8) -> PyResult<()> {
        pyresult(self.inner.react(input))
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
    fn symbols(&self) -> PyResult<usize> {
        Ok(self.inner.symbols())
    }
}

#[pyclass]
struct ParallelGameState {
    conductor: ThreadConductor,
    configs: Vec<GameConfig>,
    symbols: u8,
}

#[pymethods]
impl ParallelGameState {
    #[new]
    fn new(py: Python, max_steps: usize, configs: Vec<String>) -> PyResult<Self> {
        let configs = {
            let mut res = vec![];
            for cfg in configs {
                res.push(pyresult_with(
                    GameConfig::from_json(&cfg),
                    "Failed to parse config",
                )?);
            }
            res
        };
        let symbols = configs[0]
            .symbol_max()
            .expect("Failed to get symbol max")
            .to_byte()
            + 1;
        let cloned = configs.clone();
        let conductor = py.allow_threads(move || ThreadConductor::new(cloned, max_steps));
        let conductor = pyresult(conductor)?;
        Ok(Self {
            conductor,
            configs,
            symbols,
        })
    }
    fn screen_size(&self) -> (i32, i32) {
        (self.configs[0].height, self.configs[0].width)
    }
    fn symbols(&self) -> PyResult<usize> {
        Ok(usize::from(self.symbols))
    }
    fn seed(&mut self, py: Python, seed: Vec<u128>) -> PyResult<()> {
        let ParallelGameState {
            ref mut conductor, ..
        } = self;
        let res = py.allow_threads(move || conductor.seed(seed));
        pyresult(res)
    }
    fn states(&mut self, py: Python) -> PyResult<Vec<PlayerState>> {
        let ParallelGameState {
            ref mut conductor, ..
        } = self;
        let res = py.allow_threads(move || conductor.states());
        pyresult(res)
    }
    fn step(&mut self, py: Python, input: Vec<u8>) -> PyResult<Vec<PlayerState>> {
        let ParallelGameState {
            ref mut conductor, ..
        } = self;
        let res = py.allow_threads(move || conductor.step(input));
        pyresult(res)
    }
    fn reset(&mut self, py: Python) -> PyResult<Vec<PlayerState>> {
        let ParallelGameState {
            ref mut conductor, ..
        } = self;
        let res = py.allow_threads(move || conductor.reset());
        pyresult(res)
    }
    fn close(&mut self, py: Python) -> PyResult<()> {
        let ParallelGameState {
            ref mut conductor, ..
        } = self;
        pyresult(py.allow_threads(move || conductor.close()))
    }
}

#[cfg(unix)]
#[pyfunction]
fn replay(game: &GameState, py: Python, interval_ms: u64) -> PyResult<()> {
    use rogue_gym_devui::show_replay;
    let inputs = game.inner.runtime.saved_inputs().to_vec();
    let config = game.config.clone();
    let res = py.allow_threads(move || show_replay(config, inputs, interval_ms));
    pyresult(res)
}

#[cfg(unix)]
#[pyfunction]
fn play_cli(game: &GameState) -> PyResult<()> {
    use rogue_gym_devui::play_game;
    pyresult(play_game(game.config.clone(), false))?;
    Ok(())
}

#[pymodule]
#[pyo3(name = "_rogue_gym")]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GameState>()?;
    m.add_class::<PlayerState>()?;
    m.add_class::<ParallelGameState>()?;
    #[cfg(unix)]
    m.add_wrapped(pyo3::wrap_pyfunction!(replay))?;
    #[cfg(unix)]
    m.add_wrapped(pyo3::wrap_pyfunction!(play_cli))?;
    Ok(())
}
