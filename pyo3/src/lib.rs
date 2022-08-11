use std::str::FromStr;

use pyo3::prelude::*;
use ::zhconv::{Variant, zhconv as zhconv_plain, zhconv_mw};

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn zhconv(text: &str, target: &str, mediawiki: Option<bool>) -> String {
    let target = Variant::from_str(target).expect("Unsupported target variant");
    let mediawiki = mediawiki.unwrap_or(false);
    if mediawiki {
        zhconv_mw(text, target)
    } else {
        zhconv_plain(text, target)
    }
}

/// zhconv with GIL released when converting. It allows in-parallel conversion when used together
/// with multithreading. But be noted that it may incur non-trivial overhead due to extra FFI
/// calls for small workloads.
#[pyfunction]
fn zhconv_nogil(py: Python<'_>, text: &str, target: &str, mediawiki: Option<bool>) -> String {
    py.allow_threads(move || {
        zhconv(text, target, mediawiki)
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn zhconv_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::zhconv, m)?)?;
    m.add_function(wrap_pyfunction!(zhconv_nogil, m)?)?;
    Ok(())
}
