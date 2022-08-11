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

/// A Python module implemented in Rust.
#[pymodule]
fn pyo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(zhconv, m)?)?;
    Ok(())
}
