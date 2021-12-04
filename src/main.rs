use zhconv::{convs::*, ZhConverter};

fn main() {
    let c1 = make_converter(ZH_TW_CONV);
    let c2 = make_converter(ZH_HANT_CONV);
    let t = r#"天干物燥，小心火烛。你想干什么不干他的事。 "#;
    dbg!(c2.convert(&c1.convert(t)));
}
