use std::io::{self, Read};

use zhconv::{convs::*, ZhConverter};

// const t: &str = include_str!("../benches/data689k.txt");

fn main() {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input).unwrap();
    // dbg!(*ZH_HANT_TW_CONV);
    println!("{}", zhconv::Zh2HantConverter.convert_allowing_inline_rules(&input));
    // dbg!(ZH_HANT_TW_CONV);
    // let c1 = make_converter(ZH_TW_CONV);
    // let c2 = &zhconv::Zh2CNConverter;
    // // let t = r#"天干物燥，小心火烛。你想干什么不干他的事。公交车和出租车都是公共交通工具。老挝是一个位于东南亚的国家。"#;
    // dbg!(c2.convert(t));
}
