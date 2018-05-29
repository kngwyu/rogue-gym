#![feature(proc_macro, specialization)]

extern crate pyo3;
extern crate rogue_gym_core;
use pyo3::prelude::*;
use rogue_gym_core::{GameConfig, RunTime};

use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;
use pyo3::py::modinit as pymodinit;

#[pyclass]
struct RogueEnv {
    runtime: RunTime,
    buffer: Vec<Vec<u8>>,
}

#[pymethods]
impl RogueEnv {
    #[new]
    fn __new__(obj: &PyRawObject, config: String) -> PyResult<()> {
        obj.init(|t| {
            let config = GameConfig::from_json(&config).unwrap();
            config.build().unwrap()
        })
    }
}

#[pymodinit(_rogue_gym)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RogueEnv>()?;
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
