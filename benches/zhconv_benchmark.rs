use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zhconv::{convs::*, ZhConverter};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("load zh2Hant", |b| {
        b.iter_with_large_drop(|| make_converter(black_box(ZH_HANT_CONV)))
    });
    c.bench_function("load zh2Hans", |b| {
        b.iter_with_large_drop(|| make_converter(black_box(ZH_HANS_CONV)))
    });
    c.bench_function("load zh2TW", |b| {
        b.iter_with_large_drop(|| make_converter(black_box(ZH_TW_CONV)))
    });
    c.bench_function("load zh2HK", |b| {
        b.iter_with_large_drop(|| make_converter(black_box(ZH_HK_CONV)))
    });
    c.bench_function("load zh2CN", |b| {
        b.iter_with_large_drop(|| make_converter(black_box(ZH_CN_CONV)))
    });
    c.bench_function("load all", |b| {
        b.iter_with_large_drop(|| {
            (
                make_converter(black_box(ZH_HANS_CONV)),
                make_converter(black_box(ZH_HANT_CONV)),
                make_converter(black_box(ZH_CN_CONV)),
                make_converter(black_box(ZH_HK_CONV)),
                make_converter(black_box(ZH_TW_CONV)),
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
