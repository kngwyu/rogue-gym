#![feature(proc_macro, specialization)]

extern crate pyo3;
use pyo3::prelude::*;

use pyo3::py::modinit as pymodinit;

#[pymodinit(rogue_gym)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(a: i64, b: i64) -> PyResult<String> {
        let out = sum_as_string(a, b);
        Ok(out)
    }

    Ok(())
}

// The logic can be implemented as a normal rust function
fn sum_as_string(a: i64, b: i64) -> String {
    format!("{}", a + b).to_string()
}
