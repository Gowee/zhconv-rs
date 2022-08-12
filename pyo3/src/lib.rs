use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyString;

use pyo3_file::PyFileLikeObject;

use ::zhconv::{
    get_builtin_table, zhconv as zhconv_plain, zhconv_mw, Variant, ZhConverter as Converter,
    ZhConverterBuilder,
};

/// zhconv(text, target[, mediawiki]) -> result
///
/// Convert a text to a target Chinese variant. Converters are constructed from built-in rulesets
/// on demand and cached automatically. If `mediawiki` is `True`, inline conversion rules such as
/// `-{foo...bar}-` are activated, while converters must be rebuilt for every invocation if there
/// are global rules. Check the project's README for more info.
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

/// converter(text[, mediawiki]) -> result
///
/// Convert a text with the previously built converter. It is a callable object that works like a
/// plain function, returned by `make_converter`.
///
/// If `mediawiki` is `True`, inline conversion rules such as `-{foo...bar}-` are activated. But be
/// noted that, unlike `zhconv`, it discards silently global conversion rules such as
/// `-{H|bar...foo}-`. Check the project's README for more info.
#[pyclass]
struct ZhConverter(Converter);

#[pymethods]
impl ZhConverter {
    fn __call__(&self, py: Python<'_>, text: &str, mediawiki: Option<bool>) -> String {
        let mediawiki = mediawiki.unwrap_or(false);
        py.allow_threads(move || {
            if mediawiki {
                self.0.convert_allowing_inline_rules(text)
            } else {
                self.0.convert(text)
            }
        })
    }
}

/// make_converter(base, rules, dfa = True) -> converter
///
/// Make a converter with custom conversion rules, optionally based on a built-in ruleset
/// specified by the `base` target variant. Rules can be an array of `(from, to)` pairs, a file
/// path or a file-like object.
/// With DFA activated by default, the converter takes more time to build while converts more
/// efficiently. All built-in converters used be `zhconv` have this feature enabled for better
/// conversion performance.
///
/// The returned converter is a callable function of the type `ZhConverter`:
///
/// converter(text[, mediawiki]) -> result
///
/// Be noted that, unlike the `zhconv` function, the returned converter does not support global
/// conversion rules such as `-{H|zh-hans:foo; zh-hant:bar}-` in texts.
#[pyfunction]
fn make_converter(
    py: Python<'_>,
    base: &str,
    rules: PyObject,
    dfa: Option<bool>,
) -> PyResult<ZhConverter> {
    let mut builder = ZhConverterBuilder::new()
        .dfa(dfa.unwrap_or(true))
        .table(get_builtin_table(base.try_into().unwrap_or(Variant::Zh)));
    if let Ok(pairs) = rules.extract::<Vec<(String, String)>>(py) {
        for (from, to) in pairs.into_iter() {
            builder = builder.add_conv_pair(from, to);
        }
    } else {
        let mut text = String::new();

        if let Ok(string_ref) = rules.cast_as::<PyString>(py) {
            // path
            let path = string_ref.to_str()?; //.map_err(|_e| TypeError::new_err("Invalid Unicode encoding in file path"))?; // TODO
            File::open(path)?.read_to_string(&mut text)?;
        } else {
            // file-like
            PyFileLikeObject::with_requirements(rules, true, false, false)?
                .read_to_string(&mut text)?;
        }

        for (i, line) in text.lines().map(|line| line.trim()).enumerate() {
            if line.starts_with('#') {
                continue;
            }
            if let Some((from, to)) = line.split_once(char::is_whitespace) {
                let to = to.trim_start_matches(char::is_whitespace);
                builder = builder.add_conv_pair(from, to);
            } else {
                return Err(PyTypeError::new_err(format!(
                    "Invalid conversion rule at line {}: {}",
                    i + 1,
                    line
                )));
            }
        }
    };

    Ok(ZhConverter(builder.build()))
}

/// zhconv as in MediaWiki, oxidized with much more efficiency.
///
/// Simple usage:
/// ```python
/// from zhconv_rs import zhconv
/// assert zhconv("zh-tw", "天干物燥 小心火烛") == assert zhconv("zh-tw", "天乾物燥 小心火燭")
/// ```
#[pymodule]
fn zhconv_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::zhconv, m)?)?;
    m.add_function(wrap_pyfunction!(crate::make_converter, m)?)?;
    m.add_class::<ZhConverter>()?;
    Ok(())
}
