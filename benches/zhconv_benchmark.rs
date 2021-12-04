use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zhconv::{convs::*, ZhConverter};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("load zh2Hant", |b| b.iter(|| make_converter(black_box(ZH_HANT_CONV))));
    c.bench_function("load zh2Hans", |b| b.iter(|| make_converter(black_box(ZH_HANS_CONV))));
    c.bench_function("load zh2TW", |b| b.iter(|| make_converter(black_box(ZH_TW_CONV))));
    c.bench_function("load zh2HK", |b| b.iter(|| make_converter(black_box(ZH_HK_CONV))));
    c.bench_function("load zh2CN", |b| b.iter(|| make_converter(black_box(ZH_CN_CONV))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
