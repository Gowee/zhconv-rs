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

/// Convert a text to a target Chinese variant. Converters are constructed from built-in rulesets
/// on demand and cached automatically. If `wikitext` is `True`, inline conversion rules such as
/// `-{foo...bar}-` are activated, while converters must be rebuilt for every invocation if there
/// are global rules. Check the project's README for more info.
///
/// Supported target variants: zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY.
#[pyfunction]
#[pyo3(signature = (text, target, wikitext=true, /))]
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
    #[pyo3(signature = (text))]
    fn __call__(&self, py: Python<'_>, text: &str) -> String {
        py.allow_threads(move || self.0.convert(text))
    }
}

/// Make a converter with custom conversion pairs, optionally based on a built-in ruleset
/// specified by the `base` target variant. Pairs can be an array of `(from, to)`, a file
/// path or a file-like object that consists of space-seperated pairs line by line.
///
/// The returned converter is a callable function of the type `ZhConverter`:
///
/// converter(text) -> result
#[pyfunction]
#[pyo3(signature = (base, pairs, /))]
fn make_converter(py: Python<'_>, base: Option<&str>, pairs: PyObject) -> PyResult<ZhConverter> {
    let base = base
        .and_then(|base| base.try_into().ok())
        .unwrap_or(Variant::Zh);
    let mut builder = ZhConverterBuilder::new()
        .target(base)
        .tables(get_builtin_tables(base));
    if let Ok(pairs) = pairs.extract::<Vec<(String, String)>>(py) {
        builder = builder.conv_pairs(pairs);
    } else {
        let mut text = String::new();

        if let Ok(string_ref) = pairs.downcast_bound::<PyString>(py) {
            // path
            let path = string_ref.to_string_lossy().to_string();
            File::open(path)?.read_to_string(&mut text)?;
        } else {
            // file-like
            PyFileLikeObject::with_requirements(pairs, true, false, false, false)?
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

/// Determine whether the given text is more likely in Simplified Chinese than Traditional Chinese.
#[pyfunction]
#[pyo3(signature = (text, /))]
fn is_hans(text: &str) -> bool {
    ::zhconv::is_hans(text)
}

/// Determine the likelihood of the given text being in Simplified Chinese compared to Traditional
/// Chinese.
///
/// The return value is in the range of `[0, 1]`, where `0` indicates the text is completely in
/// Traditional Chinese, `1` indicates the text is completely in Simplified Chinese, and a value of
/// `0.5` suggests an equal proportion between the two variants. If there is no enough text to
/// determine, `NaN` is returned.
//  Doc written by ChatGPT
#[pyfunction]
#[pyo3(signature = (text, /))]
fn is_hans_confidence(text: &str) -> f32 {
    ::zhconv::is_hans_confidence(text)
}

/// Infer the Chinese character variant of the given text.
///
/// This function analyzes the text by counting matched source words with built-in converters to
/// determine which Chinese variant the text is most likely to be. Due to lack of rulesets, it
/// would never return `zh-MO`, `zh-SG`, or `zh-MY`.
///
/// The accuracy has not been assessed. Avoid relying on this for serious purposes.
//  Doc written by ChatGPT
#[pyfunction]
#[pyo3(signature = (text, /))]
fn infer_variant(text: &str) -> String {
    ::zhconv::infer_variant(text).to_string()
}

/// Infer the Chinese character variant of the given text.
///
/// This function analyzes the text by counting the matched source words using built-in converters,
/// and calculates the likelihood of the text belonging to each Chinese variant. The inferred
/// variants are returned as a vector of tuples, where each tuple contains the variant as a str and
/// its corresponding confidence level as a float.
///
/// Confidence levels are in the range of `[0, 1]`. It can be `NaN` if there is no enough text to
/// analyze.
///
/// Due to lack of rulesets, it would never return `zh-MO`, `zh-SG`, or `zh-MY`.
///
/// The confidence levels of script variants (`zh-Hant` and `zh-Hans`) are always greater than
/// those of region variants (`zh-TW`, `zh-CN` and `zh-HK`) with the current implementation.
///
/// The accuracy has not been assessed. Avoid relying on this for serious purposes.
//  Doc written by ChatGPT
#[pyfunction]
#[pyo3(signature = (text, /))]
fn infer_variant_confidence(text: &str) -> Vec<(String, f32)> {
    ::zhconv::infer_variant_confidence(text)
        .into_iter()
        .map(|(v, c)| (v.to_string(), c))
        .collect()
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
/// Check if Simp/Trad
/// ```
/// python
/// from zhconv_rs import is_hans, is_hans_confidence
/// text = "譬如鳥跡，空中現者，無有是處。"
/// assert not is_hans(text)
/// assert is_hans_confidence(text) < 0.5
/// ```
///
/// Determine the variant (experimental, low accuracy)
/// from zhconv_rs import infer_variant, infer_variant_confidence
/// text1 = "最後一班的電車落寞地駛過後 遠遠交叉路口的小紅燈熄了 但是一絮一絮濡濕了的凝固的霓虹 沾染了眼和眼之間矇矓的視覺"
/// assert infer_variant(text1).lower() in ["zh-tw", "zh-hant", "zh-hk"]
/// print(infer_variant_confidence(text1))
/// # [('zh-hant', 0.7272727489471436),
/// #   ('zh-tw', 0.7272727489471436),
/// #   ('zh-hk', 0.7272727489471436),
/// #   ('zh-hans', 0.27272725105285645),
/// #   ('zh-cn', 0.27272725105285645)]
/// text2 = "香港深受外來飲食文化影響。中環蘭桂坊、蘇豪區、灣仔及尖沙咀酒吧林立，而慕尼黑啤酒節更由1991年起每年於尖沙咀廣東道舉行；亦有不少從外地傳來的潮流飲食，如來自英國的奶茶、澳門的葡撻、台灣的珍珠奶茶、日本的壽司及美國的家鄉雞、意大利薄餅、甜品芝士蛋糕等。"
/// print(infer_variant_confidence(text1))
/// #[('zh-hant', 0.939393937587738),
/// #   ('zh-hk', 0.939393937587738),
/// #   ('zh-tw', 0.8484848737716675),
/// #   ('zh-hans', 0.15151512622833252),
/// #   ('zh-cn', 0.15151512622833252)]

#[pymodule]
fn zhconv_rs(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::zhconv, m)?)?;
    m.add_function(wrap_pyfunction!(crate::make_converter, m)?)?;
    m.add_function(wrap_pyfunction!(crate::is_hans, m)?)?;
    m.add_function(wrap_pyfunction!(crate::is_hans_confidence, m)?)?;
    m.add_function(wrap_pyfunction!(crate::infer_variant, m)?)?;
    m.add_function(wrap_pyfunction!(crate::infer_variant_confidence, m)?)?;
    m.add_class::<ZhConverter>()?;
    Ok(())
}
