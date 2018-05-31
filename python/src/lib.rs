#![feature(proc_macro, specialization)]

extern crate pyo3;
extern crate rect_iter;
extern crate rogue_gym_core;
use pyo3::prelude::*;
use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;
use pyo3::py::modinit as pymodinit;
use pyo3::{IntoPyDictPointer, PyBytes, PyList};
use rect_iter::GetMut2D;
use rogue_gym_core::character::player::Status;
use rogue_gym_core::dungeon::{Positioned, X, Y};
use rogue_gym_core::error::{GameResult, ResultExt};
use rogue_gym_core::{
    input::{Key, KeyMap}, GameConfig, Reaction, RunTime,
};

#[pyclass]
struct GameState {
    runtime: RunTime,
    state: PlayerState,
    config: GameConfig,
    token: PyToken,
}

/// result of the action(map as list of byte array, status as dict)
type ActionResult<'p> = (&'p PyList, PyObject);

#[derive(Debug, Clone)]
struct PlayerState {
    map: Vec<Vec<u8>>,
    status: Status,
}

impl PlayerState {
    fn new(w: X, h: Y) -> Self {
        PlayerState {
            map: vec![vec![b' '; w.0 as usize]; h.0 as usize],
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
    fn res<'p>(&self, py: Python<'p>) -> ActionResult<'p> {
        let map: Vec<_> = self.map.iter().map(|v| PyBytes::new(py, &v)).collect();
        let map = PyList::new(py, &map);
        let status = self.status.to_vec().into_dict_ptr(py);
        let status = unsafe { PyObject::from_owned_ptr(py, status) };
        (map, status)
    }
}

#[pymethods]
impl GameState {
    #[new]
    fn __new__(obj: &PyRawObject, config: Option<String>) -> PyResult<()> {
        let config = config.map_or_else(GameConfig::default, |cfg| {
            GameConfig::from_json(&cfg).unwrap()
        });
        let mut runtime = config.clone().build().unwrap();
        let (w, h) = runtime.screen_size();
        let mut state = PlayerState::new(w, h);
        runtime.keymap = KeyMap::ai();
        state.update(&mut runtime).unwrap();
        obj.init(|token| GameState {
            runtime,
            state,
            config,
            token,
        })
    }
    fn set_seed(&mut self, seed: u64) -> PyResult<()> {
        self.config.seed = Some(seed);
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
        Ok(self.state.res(self.token.py()))
    }
    fn react(&mut self, input: u8) -> PyResult<ActionResult> {
        let res = self.runtime.react_to_key(Key::Char(input as char));
        let res = match res {
            Ok(ok) => ok,
            Err(e) => {
                if e.kind().can_allow() {
                    return Ok(self.state.res(self.token.py()));
                }
                panic!("error in game: {}", e);
            }
        };
        res.into_iter().for_each(|reaction| match reaction {
            Reaction::Redraw => {
                self.state.draw_map(&self.runtime).unwrap();
            }
            Reaction::StatusUpdated => {
                self.state.status = self.runtime.player_status();
            }
            Reaction::UiTransition(_) => {}
            Reaction::Notify(_) => {}
        });
        Ok(self.state.res(self.token.py()))
    }
}

#[pymodinit(_rogue_gym)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GameState>()?;
    Ok(())
}
