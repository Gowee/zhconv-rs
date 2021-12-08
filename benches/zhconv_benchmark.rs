use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zhconv::{convs::*, ZhConverter};

const DATA54K: &'static str = include_str!("data54k.txt");
const DATA689K: &'static str = include_str!("data689k.txt");
const DATA3185K: &'static str = include_str!("data3185k.txt");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("build zh-Hant-HK", |b| {
        b.iter_with_large_drop(|| merge_convs(black_box(ZH_HANT_CONV), black_box(ZH_HK_CONV)))
    });
    c.bench_function("build zh-Hant-MO", |b| {
        b.iter_with_large_drop(|| merge_convs(black_box(ZH_HANT_CONV), black_box(ZH_MO_CONV)))
    });
    c.bench_function("build zh-Hans-CN", |b| {
        b.iter_with_large_drop(|| merge_convs(black_box(ZH_HANS_CONV), black_box(ZH_CN_CONV)))
    });
    c.bench_function("build zh-Hans-SG", |b| {
        b.iter_with_large_drop(|| merge_convs(black_box(ZH_HANS_CONV), black_box(ZH_SG_CONV)))
    });
    c.bench_function("build zh-Hans-MY", |b| {
        b.iter_with_large_drop(|| merge_convs(black_box(ZH_HANS_CONV), black_box(ZH_MY_CONV)))
    });
    c.bench_function("load zh2Hant", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(ZH_HANT_CONV)))
    });
    c.bench_function("load zh2Hans", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(ZH_HANS_CONV)))
    });
    c.bench_function("load zh2TW", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(*ZH_HANT_TW_CONV)))
    });
    c.bench_function("load zh2HK", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(*ZH_HANT_HK_CONV)))
    });
    c.bench_function("load zh2MO", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(*ZH_HANT_MO_CONV)))
    });
    c.bench_function("load zh2CN", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(*ZH_HANS_CN_CONV)))
    });
    c.bench_function("load zh2SG", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(*ZH_HANS_SG_CONV)))
    });
    c.bench_function("load zh2MY", |b| {
        b.iter_with_large_drop(|| build_converter(black_box(*ZH_HANS_MY_CONV)))
    });
    // c.bench_function("load all", |b| {
    //     b.iter_with_large_drop(|| {
    //         (
    //             make_converter(black_box(ZH_HANS_CONV)),
    //             make_converter(black_box(ZH_HANT_CONV)),
    //             make_converter(black_box(ZH_CN_CONV)),
    //             make_converter(black_box(ZH_HK_CONV)),
    //             make_converter(black_box(ZH_TW_CONV)),
    //         )
    //     })
    // });

    c.bench_function("zh2TW data54k", |b| {
        b.iter_with_large_drop(|| zhconv::Zh2TWConverter.convert(DATA54K))
    });
    c.bench_function("zh2CN data54k", |b| {
        b.iter_with_large_drop(|| zhconv::Zh2CNConverter.convert(DATA54K))
    });
    c.bench_function("zh2Hant data689k", |b| {
        b.iter_with_large_drop(|| zhconv::Zh2HantConverter.convert(DATA689K))
    });
    c.bench_function("zh2TW data689k", |b| {
        b.iter_with_large_drop(|| zhconv::Zh2TWConverter.convert(DATA689K))
    });
    c.bench_function("zh2Hant data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::Zh2HantConverter.convert(DATA3185K))
    });
    c.bench_function("zh2TW data3185k", |b| {
        b.iter_with_large_drop(|| zhconv::Zh2TWConverter.convert(DATA3185K))
    });
}

criterion_group!(benches, criterion_benchmark);
// criterion_group! {
//     name = benches;
//     config = Criterion::default().sample_size(500);
//     targets = criterion_benchmark
// }
criterion_main!(benches);
