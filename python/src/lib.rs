#![feature(proc_macro, specialization)]

extern crate pyo3;
extern crate rogue_gym_core;
use rogue_gym_core::{GameConfig, RunTime, dungeon::{X, Y}, input::{Key, KeyMap}};
use rogue_gym_core::character::player::Status;

use pyo3::prelude::*;
use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;
use pyo3::py::modinit as pymodinit;

#[pyclass]
struct GameState {
    runtime: RunTime,
    state: PlayerState,
    token: PyToken,
}

#[pyclass]
#[derive(Clone, Debug)]
struct PlayerState {
    map: Vec<Vec<u8>>,
    status: Status,
}

impl PlayerState {
    fn new(w: X, h: Y) -> Self {
        PlayerState {
            map: vec![vec![0; w.0 as usize]; h.0 as usize],
            status: Status::default(),
        }
    }
}

#[pymethods]
impl GameState {
    #[new]
    fn __new__(obj: &PyRawObject, config: Option<String>) -> PyResult<()> {
        let config = config.map_or_else(GameConfig::default, |cfg| {
            GameConfig::from_json(&cfg).unwrap()
        });
        let mut runtime = config.build().unwrap();
        let (w, h) = runtime.screen_size();
        runtime.keymap = KeyMap::ai();
        obj.init(|token| GameState {
            runtime,
            state: PlayerState::new(w, h),
            token,
        })
    }
    fn react(&mut self, input: u8) -> PyResult<Vec<Vec<u8>>> {
        let res = self.runtime.react_to_key(Key::Char(input as char));
        let res = match res {
            Ok(ok) => ok,
            Err(e) => {
                if e.kind().can_allow() {
                    return Ok(self.state.map.clone());
                }
                panic!("error in game: {}", e);
            }
        };
        Ok(self.state.map.clone())
    }
}

#[pymodinit(_rogue_gym)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GameState>()?;
    #[pyfn(m, "sum_as_str")]
    fn sum_as_str(a: i64, b: i64) -> PyResult<String> {
        let out = sum_as_string(a, b);
        Ok(out)
    }
    Ok(())
}

// The logic can be implemented as a normal rust function
fn sum_as_string(a: i64, b: i64) -> String {
    format!("{}", a + b).to_string()
}
