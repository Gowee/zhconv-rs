use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyString;

use pyo3_file::PyFileLikeObject;

use ::zhconv::{
    get_builtin_tables, zhconv as zhconv_plain, zhconv_mw, Variant, ZhConverter as Converter,
    ZhConverterBuilder,
};

/// zhconv(text, target[, wikitext]) -> result
///
/// Convert a text to a target Chinese variant. Converters are constructed from built-in rulesets
/// on demand and cached automatically. If `wikitext` is `True`, inline conversion rules such as
/// `-{foo...bar}-` are activated, while converters must be rebuilt for every invocation if there
/// are global rules. Check the project's README for more info.
///
/// Supported target variants: zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY.
#[pyfunction]
fn zhconv(py: Python<'_>, text: &str, target: &str, wikitext: Option<bool>) -> PyResult<String> {
    py.allow_threads(move || {
        let target = Variant::from_str(target).map_err(|_e| {
            PyTypeError::new_err(format!("Unsupported target variant: {}", target))
        })?;
        let wikitext = wikitext.unwrap_or(false);
        Ok(if wikitext {
            zhconv_mw(text, target)
        } else {
            zhconv_plain(text, target)
        })
    })
}

/// converter(text) -> result
///
/// Convert a text with the previously built converter. It is a callable object that works like a
/// plain function, returned by `make_converter`.
#[pyclass]
struct ZhConverter(Converter);

#[pymethods]
impl ZhConverter {
    fn __call__(&self, py: Python<'_>, text: &str) -> String {
        py.allow_threads(move || self.0.convert(text))
    }
}

/// make_converter(base, rules) -> converter
///
/// Make a converter with custom conversion rules, optionally based on a built-in ruleset
/// specified by the `base` target variant. Rules can be an array of `(from, to)` pairs, a file
/// path or a file-like object that consists of space-seperated pairs line by line.
///
/// The returned converter is a callable function of the type `ZhConverter`:
///
/// converter(text) -> result
#[pyfunction]
fn make_converter(
    py: Python<'_>,
    base: Option<&str>,
    rules: PyObject,
) -> PyResult<ZhConverter> {
    let base = base
        .and_then(|base| base.try_into().ok())
        .unwrap_or(Variant::Zh);
    let mut builder = ZhConverterBuilder::new()
        .target(base)
        .tables(get_builtin_tables(base));
    if let Ok(pairs) = rules.extract::<Vec<(String, String)>>(py) {
        builder = builder.conv_pairs(pairs.into_iter());
    } else {
        let mut text = String::new();

        if let Ok(string_ref) = rules.cast_as::<PyString>(py) {
            // path
            let path = string_ref.to_str()?;
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
                builder = builder.conv_pairs([(from, to)]);
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
/// Convert with builtin rulesets:
/// ```python
/// from zhconv_rs import zhconv
/// assert zhconv("天干物燥 小心火烛", "zh-tw") == "天乾物燥 小心火燭"
/// assert zhconv("《-{zh-hans:三个火枪手;zh-hant:三劍客;zh-tw:三劍客}-》是亞歷山大·仲馬的作品。", "zh-cn", wikitext=True) == "《三个火枪手》是亚历山大·仲马的作品。"
/// assert zhconv("-{H|zh-cn:雾都孤儿;zh-tw:孤雛淚;zh-hk:苦海孤雛;zh-sg:雾都孤儿;zh-mo:苦海孤雛;}-《雾都孤儿》是查尔斯·狄更斯的作品。", "zh-tw", True) == "《孤雛淚》是查爾斯·狄更斯的作品。"
/// ```
///
/// Convert with custom rules:
/// ```python
/// from zhconv_rs import make_converter
/// assert make_converter(None, [("天", "地"), ("水", "火")])("甘肅天水") == "甘肅地火"
///
/// import io
/// convert = make_converter("zh-hans", io.StringIO("䖏 处\n罨畫 掩画"))
/// assert convert("秀州西去湖州近 幾䖏樓臺罨畫間") == "秀州西去湖州近 几处楼台掩画间"
///
/// "譬如鳥跡，空中現者，無有是處。"
/// ```
#[pymodule]
fn zhconv_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::zhconv, m)?)?;
    m.add_function(wrap_pyfunction!(crate::make_converter, m)?)?;
    m.add_class::<ZhConverter>()?;
    Ok(())
}
