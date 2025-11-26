use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use zhconv::{converters::deserialize_converter, tables::*, Variant};

const WIKITEXT: &str = include_str!("wikitext.txt");
const DATA54K: &str = include_str!("data54k.txt");
const DATA689K: &str = include_str!("data689k.txt");
const DATA3185K: &str = include_str!("data3185k.txt");

const CONVTUPLES: [(&str, Variant); 8] = [
    ("zh2Hant", Variant::ZhHant),
    ("zh2Hans", Variant::ZhHans),
    ("zh2TW", Variant::ZhTW),
    ("zh2HK", Variant::ZhHK),
    ("zh2MO", Variant::ZhMO),
    ("zh2CN", Variant::ZhCN),
    ("zh2SG", Variant::ZhSG),
    ("zh2MY", Variant::ZhMY),
];

fn bench_load(c: &mut Criterion) {
    // c.bench_function("build zh-Hant-HK", |b| {
    //     b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANT_TABLE), black_box(ZH_HK_TABLE)))
    // });
    // c.bench_function("build zh-Hant-MO", |b| {
    //     b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANT_TABLE), black_box(ZH_MO_TABLE)))
    // });
    // c.bench_function("build zh-Hans-CN", |b| {
    //     b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANS_TABLE), black_box(ZH_CN_TABLE)))
    // });
    // c.bench_function("build zh-Hans-SG", |b| {
    //     b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANS_TABLE), black_box(ZH_SG_TABLE)))
    // });
    // c.bench_function("build zh-Hans-MY", |b| {
    //     b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANS_TABLE), black_box(ZH_MY_TABLE)))
    // });

    // c.bench_function("test", |b| {
    //     b.iter_with_large_drop(|| {
    //         black_box(zhconv::tables::daac())
    //     })
    // });//lz4_flex::compress_prepend_size
    {
        // let mut build = c.benchmark_group("build-from-scratch");
        // for (name, variant) in CONVTUPLES {
        //     build.bench_function(name, |b| {
        //         b.iter_with_large_drop(|| {
        //             build_converter(variant, black_box(get_builtin_table(variant)))
        //         })
        //     });
        // }

        // c.bench_function("build all from scratch", |b| {
        //     b.iter_with_large_drop(|| {
        //         (
        //             make_converter(black_box(ZH_HANS_TABLE)),
        //             make_converter(black_box(ZH_HANT_TABLE)),
        //             make_converter(black_box(ZH_CN_TABLE)),
        //             make_converter(black_box(ZH_HK_TABLE)),
        //             make_converter(black_box(ZH_TW_TABLE)),
        //         )
        //     })
        // });
    }
    let group_name = match (
        cfg!(feature = "opencc"),
        cfg!(feature = "opencc-twp"),
        cfg!(feature = "compress"),
    ) {
        (_, true, true) => "load-opencc-twp-compressed-serialized",
        (_, true, false) => "load-opencc-twp-serialized",
        (true, false, true) => "load-opencc-compressed-serialized",
        (true, false, false) => "load-opencc-serialized",
        (false, _, true) => "load-compressed-serialized",
        (false, _, false) => "load-serialized",
    };

    let mut load = c.benchmark_group(group_name);
    load.sample_size(20);
    for (name, variant) in CONVTUPLES {
        load.bench_function(name, |b| {
            b.iter_with_large_drop(|| {
                deserialize_converter(
                    variant,
                    black_box(get_builtin_serialized_daac(variant)),
                    black_box(get_builtin_tables(variant).iter().cloned()),
                )
            })
        });
    }
}

fn bench_convert(c: &mut Criterion) {
    let group_name = match (cfg!(feature = "opencc"), cfg!(feature = "opencc-twp")) {
        (_, true) => "convert-opencc-twp",
        (true, false) => "convert-opencc",
        (false, false) => "convert",
    };

    let mut convert = c.benchmark_group(group_name);
    convert.sample_size(20);
    convert.bench_function("zh2CN wikitext basic", |b| {
        b.iter_with_large_drop(|| {
            zhconv::converters::ZH_TO_CN_CONVERTER.convert_as_wikitext_basic(WIKITEXT)
        })
    });
    convert.bench_function("zh2TW wikitext basic", |b| {
        b.iter_with_large_drop(|| {
            zhconv::converters::ZH_TO_TW_CONVERTER.convert_as_wikitext_basic(WIKITEXT)
        })
    });
    convert.bench_function("zh2TW wikitext extended", |b| {
        b.iter_with_large_drop(|| {
            zhconv::converters::ZH_TO_TW_CONVERTER.convert_as_wikitext_extended(WIKITEXT)
        })
    });
    convert.bench_function("zh2CN 天乾物燥", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_CN_CONVERTER.convert("天乾物燥"))
    });
    convert.bench_function("zh2TW data54k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_TW_CONVERTER.convert(DATA54K))
    });
    convert.bench_function("zh2CN data54k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_CN_CONVERTER.convert(DATA54K))
    });
    convert.bench_function("zh2Hant data689k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_HANT_CONVERTER.convert(DATA689K))
    });
    convert.bench_function("zh2TW data689k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_TW_CONVERTER.convert(DATA689K))
    });
    convert.bench_function("zh2Hant data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_HANT_CONVERTER.convert(DATA3185K))
    });
    convert.bench_function("zh2TW data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_TW_CONVERTER.convert(DATA3185K))
    });
    convert.bench_function("zh2TW data55m", |b| {
        b.iter_with_large_drop(|| {
            zhconv::converters::ZH_TO_TW_CONVERTER.convert(
                &(String::from(DATA3185K)
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K
                    + DATA3185K),
            )
        })
    });
}

fn bench_utils(c: &mut Criterion) {
    let mut utils = c.benchmark_group("utils");
    utils.sample_size(20);
    utils.bench_function("is_hans data55k", |b| {
        b.iter_with_large_drop(|| zhconv::is_hans(DATA54K))
    });
    utils.bench_function("infer_variant data55k", |b| {
        b.iter_with_large_drop(|| zhconv::infer_variant(DATA54K))
    });
    utils.bench_function("is_hans data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::is_hans(DATA3185K))
    });
    utils.bench_function("infer_variant data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::infer_variant(DATA3185K))
    });
}

criterion_group!(benches, bench_load, bench_convert, bench_utils);
// criterion_group! {
//     name = benches;
//     config = Criterion::default().sample_size(500);
//     targets = criterion_benchmark
// }
criterion_main!(benches);
