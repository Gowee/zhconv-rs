use zhconv::{convs::*, ZhConverter};

fn main() {
    // dbg!(ZH_HANT_TW_CONV);
    // let c1 = make_converter(ZH_TW_CONV);
    let c2 = make_converter(*ZH_HANT_HK_CONV);
    let t = r#"天干物燥，小心火烛。你想干什么不干他的事。公交车和出租车都是公共交通工具。老挝是一个位于东南亚的国家。"#;
    dbg!(c2.convert(t));
}
