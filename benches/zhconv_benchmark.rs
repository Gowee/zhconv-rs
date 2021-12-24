use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zhconv::{tables::*, Variant};

const DATA54K: &str = include_str!("data54k.txt");
const DATA689K: &str = include_str!("data689k.txt");
const DATA3185K: &str = include_str!("data3185k.txt");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("build zh-Hant-HK", |b| {
        b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANT_TABLE), black_box(ZH_HK_TABLE)))
    });
    c.bench_function("build zh-Hant-MO", |b| {
        b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANT_TABLE), black_box(ZH_MO_TABLE)))
    });
    c.bench_function("build zh-Hans-CN", |b| {
        b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANS_TABLE), black_box(ZH_CN_TABLE)))
    });
    c.bench_function("build zh-Hans-SG", |b| {
        b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANS_TABLE), black_box(ZH_SG_TABLE)))
    });
    c.bench_function("build zh-Hans-MY", |b| {
        b.iter_with_large_drop(|| merge_tables(black_box(ZH_HANS_TABLE), black_box(ZH_MY_TABLE)))
    });
    c.bench_function("load zh2Hant", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(ZH_HANT_TABLE)))
    });
    c.bench_function("load zh2Hans", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(ZH_HANS_TABLE)))
    });
    c.bench_function("load zh2TW", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(*ZH_HANT_TW_TABLE)))
    });
    c.bench_function("load zh2HK", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(*ZH_HANT_HK_TABLE)))
    });
    c.bench_function("load zh2MO", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(*ZH_HANT_MO_TABLE)))
    });
    c.bench_function("load zh2CN", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(*ZH_HANS_CN_TABLE)))
    });
    c.bench_function("load zh2SG", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(*ZH_HANS_SG_TABLE)))
    });
    c.bench_function("load zh2MY", |b| {
        b.iter_with_large_drop(|| build_converter(Variant::Zh, black_box(*ZH_HANS_MY_TABLE)))
    });
    // c.bench_function("load all", |b| {
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

    c.bench_function("zh2TW data54k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_TW_CONVERTER.convert(DATA54K))
    });
    c.bench_function("zh2CN data54k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_CN_CONVERTER.convert(DATA54K))
    });
    c.bench_function("zh2Hant data689k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_HANT_CONVERTER.convert(DATA689K))
    });
    c.bench_function("zh2TW data689k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_TW_CONVERTER.convert(DATA689K))
    });
    c.bench_function("zh2Hant data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_HANT_CONVERTER.convert(DATA3185K))
    });
    c.bench_function("zh2TW data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::converters::ZH_TO_TW_CONVERTER.convert(DATA3185K))
    });
    c.bench_function("zh2TW data55m", |b| {
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

criterion_group!(benches, criterion_benchmark);
// criterion_group! {
//     name = benches;
//     config = Criterion::default().sample_size(500);
//     targets = criterion_benchmark
// }
criterion_main!(benches);
