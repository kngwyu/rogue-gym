#![feature(specialization)]
extern crate numpy;
extern crate pyo3;
extern crate rect_iter;
extern crate rogue_gym_core;

use numpy::{PyArray, PyArrayModule};
use pyo3::{exc, prelude::*};
use rect_iter::GetMut2D;
use rogue_gym_core::character::player::Status;
use rogue_gym_core::dungeon::{Positioned, X, Y};
use rogue_gym_core::error::*;
use rogue_gym_core::tile::construct_symbol_map;
use rogue_gym_core::{
    input::{Key, KeyMap},
    GameConfig, Reaction, RunTime,
};

/// result of the action
/// (map as list of byte array, status as dict, status to display, feature map)
type ActionResult<'p> = (&'p PyList, &'p PyDict, Py<PyString>, Option<PyArray<f32>>);

#[derive(Debug)]
struct PlayerState {
    map: Vec<Vec<u8>>,
    status: Status,
}

impl PlayerState {
    fn new(w: X, h: Y) -> Self {
        let (w, h) = (w.0 as usize, h.0 as usize);
        PlayerState {
            map: vec![vec![b' '; w]; h],
            status: Status::default(),
        }
    }
    fn update(&mut self, runtime: &RunTime) -> GameResult<()> {
        self.status = runtime.player_status();
        self.draw_map(runtime)
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
        for (k, v) in self.status.to_vec() {
            status.set_item(k, v)?;
        }
        let status_str = PyString::new(py, &format!("{}", self.status));
        let np = PyArrayModule::import(py)?;
        let sym_map = construct_symbol_map(&self.map).and_then(|mut v| {
            let (w, h) = (v[0][0].len(), v[0].len());
            v.extend(self.status.to_image(w, h));
            PyArray::from_vec3(py, &np, &v).ok()
        });
        Ok((map, status, status_str, sym_map))
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
        let mut state = PlayerState::new(w, h);
        state.update(&mut runtime).unwrap();
        obj.init(|token| GameState {
            runtime,
            state,
            config,
            prev_actions: vec![Reaction::Redraw],
            token,
        })
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
    Ok(())
}
