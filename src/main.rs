use std::io::{self, Read};

use zhconv::{get_builtin_table, Variant, ZhConverterBuilder};

// const t: &str = include_str!("../benches/data689k.txt");

fn main() {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input).unwrap();
    // dbg!(*ZH_HANT_TW_TABLE);s
    println!("{}", ZhConverterBuilder::new()
    .target(Variant::ZhTW)
    .table(get_builtin_table(Variant::ZhTW))
    .conv_lines("zh-cn:人工智能; zh-hk:人工智能; zh-tw:人工智慧;\nzh:訪問; zh-cn:访问; zh-tw:存取;\nzh-cn:访问控制表;zh-tw:存取控制串列\nzh-cn:接入点;\n")
    .rules_from_page(&input)
    .dfa(false)
    .build()
    .convert(&input));
    // dbg!(ZH_HANT_TW_TABLE);
    // let c1 = make_converter(ZH_TW_TABLE);
    // let c2 = &zhconv::ZH_TO_CN_CONVERTER;
    // // let t = r#"天干物燥，小心火烛。你想干什么不干他的事。公交车和出租车都是公共交通工具。老挝是一个位于东南亚的国家。"#;
    //天干物燥，小心火烛。你想干什么不干他的事。天干地支，简称干支，是传统纪年方法。公交车和出租车都是公共交通工具。老挝（-{D|zh-cn:老挝; zh-hk: 寮國}-）是一个位于东南亚的国家。
    // dbg!(c2.convert(t));
}
