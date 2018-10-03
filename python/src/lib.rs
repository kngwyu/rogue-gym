#![feature(specialization)]
extern crate ndarray;
extern crate numpy;
extern crate pyo3;
extern crate rect_iter;
extern crate rogue_gym_core;

use ndarray::{Array2, Array3};
use numpy::PyArray;
use pyo3::{exc, prelude::*};
use rect_iter::{GetMut2D, RectRange};
use rogue_gym_core::character::player::Status;
use rogue_gym_core::dungeon::{Positioned, X, Y};
use rogue_gym_core::error::*;
use rogue_gym_core::symbol;
use rogue_gym_core::{
    input::{Key, KeyMap},
    GameConfig, Reaction, RunTime,
};

/// result of the action
/// (map as list of byte array, status as dict, status to display, feature map)
type ActionResult<'p> = (&'p PyList, &'p PyDict, Py<PyString>, PlayerState);

/// Memory efficient representation of State.
#[pyclass]
#[derive(Clone, Debug)]
struct PlayerState {
    symbol_map: Array2<u8>,
    channels: u8,
    status: Box<[u32]>,
}

impl PlayerState {
    fn new(
        map: &Vec<Vec<u8>>,
        stats: &Status,
        channels: u8,
    ) -> Result<Self, symbol::InvalidTileError> {
        let mut syms = Array2::zeros([map.len(), map[0].len()]);
        symbol::construct_symbol_map(map, map.len(), map[0].len(), channels, &mut syms)?;
        let status = stats.to_vec().into_boxed_slice();
        Ok(PlayerState {
            symbol_map: syms,
            status,
            channels,
        })
    }
}

impl ::pyo3::IntoPyObject for PlayerState {
    fn into_object(self, py: ::pyo3::Python) -> ::pyo3::PyObject {
        ::pyo3::Py::new(py, |_| self).unwrap().into_object(py)
    }
}

#[pymethods]
impl PlayerState {
    fn to_symbol_image(&self) -> PyArray<f32> {}
}

#[derive(Debug)]
struct PlayerStateInner {
    map: Vec<Vec<u8>>,
    range: RectRange<usize>,
    feature_buf: Array3<f32>,
    channels: u8,
    status: Status,
}

impl PlayerStateInner {
    fn new(w: X, h: Y, channels: u8) -> Self {
        let (w, h) = (w.0 as usize, h.0 as usize);
        PlayerStateInner {
            map: vec![vec![b' '; w]; h],
            range: RectRange::zero_start(w, h).unwrap(),
            feature_buf: Array3::zeros([usize::from(channels), h, w]),
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
        runtime.draw_screen(|Positioned(cd, tile)| -> GameResult<()> {
            *self
                .map
                .try_get_mut_p(cd)
                .into_chained(|| "in python::GameState::react")? = tile.to_byte();
            Ok(())
        })
    }
    fn res<'p>(&self, py: Python<'p>) -> PyResult<ActionResult<'p>> {
        let map: Vec<_> = self.map.iter().map(|v| PyBytes::new(py, &v)).collect();
        let map = PyList::new(py, &map);
        let status = PyDict::new(py);
        for (k, v) in self.status.to_dict_vec() {
            status.set_item(k, v)?;
        }
        let status_str = PyString::new(py, &format!("{}", self.status));
        let player_status =
            PlayerState::new(&self.map, &self.status, self.channels).map_err(|e| {
                PyErr::new::<exc::RuntimeError, _>(format!("error in rogue_gym_core: {}", e))
            })?;
        Ok((map, status, status_str, player_status))
    }
}

#[pyclass]
struct GameState {
    runtime: RunTime,
    state: PlayerStateInner,
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
        let mut state = PlayerStateInner::new(w, h, channels);
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
    fn reset(&mut self) -> PyResult<()> {
        let mut runtime = self.config.clone().build().unwrap();
        runtime.keymap = KeyMap::ai();
        self.state.update(&mut runtime).unwrap();
        self.runtime = runtime;
        Ok(())
    }
    fn prev(&self) -> PyResult<ActionResult> {
        self.state.res(self.token.py())
    }
    fn react(&mut self, input: u8) -> PyResult<ActionResult> {
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
        self.state.res(self.token.py())
    }
}

#[pymodinit(_rogue_gym)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GameState>()?;
    m.add_class::<PlayerState>()?;
    Ok(())
}
