use std::str::FromStr;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use ::zhconv::{zhconv as zhconv_plain, zhconv_mw, Variant};

/// zhconv(text, target[, mediawiki]) -> result
///
/// Convert the text to a target Chinese variant. Converters are constructed from built-in rulesets
/// on demand and cached automatically. If `mediawiki` is `True`, inline conversion rules such as
/// `-{foobar}-` are activated, while converters must be rebuilt for every invocation if there are
/// global rules. Check the project's README for more info.
///
/// Supported target variants: zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY.
#[pyfunction]
fn zhconv(py: Python<'_>, text: &str, target: &str, mediawiki: Option<bool>) -> PyResult<String> {
    py.allow_threads(move || {
        let target = Variant::from_str(target)
            .map_err(|_e| PyTypeError::new_err("Unsupported target variant"))?;
        let mediawiki = mediawiki.unwrap_or(false);
        Ok(if mediawiki {
            zhconv_mw(text, target)
        } else {
            zhconv_plain(text, target)
        })
    })
}

/// zhconv as in MediaWiki, oxidized with much more efficiency.
#[pymodule]
fn zhconv_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::zhconv, m)?)?;
    Ok(())
}
