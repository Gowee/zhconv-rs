use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

const DATA3185K: &str = include_str!("data3185k.txt");

fn bench_compare_to_tw(c: &mut Criterion) {
    let data = zhconv::zhconv(DATA3185K, zhconv::Variant::ZhCN);
    let opencc = cfg!(feature = "opencc");
    let twp = cfg!(feature = "opencc-twp");
    // let group_name = if twp {
    //     "compare-bench-to-opencc-twp-3185K"
    // } else if opencc {
    //     "compare-bench-to-opencc-tw-3185K"
    // } else {
    //     "compare-bench-to-tw-3185K"
    // };
    let mut group = c.benchmark_group("compare-bench-to-tw-3185K");
    group.sample_size(20);

    // zhconv-rs
    // adjust the call below to match zhconv-rs API if needed
    group.bench_function(
        if twp {
            "zhconv-rs-opencc-twp"
        } else if opencc {
            "zhconv-rs-opencc"
        } else {
            "zhconv-rs"
        },
        |b| {
            b.iter_with_large_drop(|| {
                let _result = zhconv::zhconv(&data, zhconv::Variant::ZhTW);
                // ensure result is not optimized away
                black_box(&_result);
            })
        },
    );

    // // fast2s (no 2tw support)
    // // adjust the call below to match fast2s API if needed
    // group.bench_function("fast2s", |b| {
    //     b.iter_with_large_drop(|| {
    //         let _result = fast2s::convert(&data);
    //         black_box(&_result);
    //     })
    // });

    if opencc {
        // opencc-rust
        // adjust the OpenCC construction / convert call to match opencc-rust API
        let opencc = opencc_rust::OpenCC::new(if twp {
            opencc_rust::DefaultConfig::S2TWP
        } else {
            opencc_rust::DefaultConfig::S2TW
        })
        .expect("create OpenCC");
        group.bench_function(
            if twp {
                "opencc-rust-twp"
            } else {
                "opencc-rust"
            },
            |b| {
                b.iter_with_large_drop(|| {
                    let _result = opencc.convert(&data);
                    black_box(&_result);
                })
            },
        );

        // ferrous-opencc
        // adjust to the ferrous-opencc API if different
        let ferrous = ferrous_opencc::OpenCC::from_config(if twp {
            ferrous_opencc::config::BuiltinConfig::S2twp
        } else {
            ferrous_opencc::config::BuiltinConfig::S2tw
        })
        .expect("create ferrous OpenCC");
        group.bench_function(
            if twp {
                "ferrous-opencc-twp"
            } else {
                "ferrous-opencc"
            },
            |b| {
                b.iter_with_large_drop(|| {
                    let _result = ferrous.convert(&data);
                    black_box(&_result);
                })
            },
        );

        // opencc-fmmseg
        let mut opencc_fmmseg = opencc_fmmseg::OpenCC::new();
        group.bench_function(
            if twp {
                "opencc-fmmseg-twp-rayon"
            } else {
                "opencc-fmmseg-rayon"
            },
            |b| {
                b.iter_with_large_drop(|| {
                    let _result =
                        opencc_fmmseg.convert(&data, if twp { "s2twp" } else { "s2tw" }, false);
                    black_box(&_result);
                })
            },
        );
        group.bench_function(
            if twp {
                "opencc-fmmseg-twp-no-rayon"
            } else {
                "opencc-fmmseg-no-rayon"
            },
            |b| {
                opencc_fmmseg.set_parallel(false);
                b.iter_with_large_drop(|| {
                    let _result =
                        opencc_fmmseg.convert(&data, if twp { "s2twp" } else { "s2tw" }, false);
                    black_box(&_result);
                })
            },
        );
    }

    group.finish();
}

fn bench_compare_to_hans(c: &mut Criterion) {
    let data = zhconv::zhconv(DATA3185K, zhconv::Variant::ZhTW);
    let opencc = cfg!(feature = "opencc");
    let mut group = c.benchmark_group("compare-bench-to-hans-3185K");
    group.sample_size(20);

    // zhconv-rs
    // adjust the call below to match zhconv-rs API if needed
    group.bench_function(
        if opencc {
            "zhconv-rs-opencc"
        } else {
            "zhconv-rs"
        },
        |b| {
            b.iter_with_large_drop(|| {
                let _result = zhconv::zhconv(&data, zhconv::Variant::ZhHans);
                // ensure result is not optimized away
                black_box(&_result);
            })
        },
    );

    // fast2s
    // adjust the call below to match fast2s API if needed
    group.bench_function("fast2s", |b| {
        b.iter_with_large_drop(|| {
            let _result = fast2s::convert(&data);
            black_box(&_result);
        })
    });

    if opencc {
        // opencc-rust
        // adjust the OpenCC construction / convert call to match opencc-rust API
        let opencc =
            opencc_rust::OpenCC::new(opencc_rust::DefaultConfig::T2S).expect("create OpenCC");
        group.bench_function("opencc-rust", |b| {
            b.iter_with_large_drop(|| {
                let _result = opencc.convert(&data);
                black_box(&_result);
            })
        });

        // ferrous-opencc
        // adjust to the ferrous-opencc API if different
        let ferrous =
            ferrous_opencc::OpenCC::from_config(ferrous_opencc::config::BuiltinConfig::T2s)
                .expect("create ferrous OpenCC");
        group.bench_function("ferrous-opencc", |b| {
            b.iter_with_large_drop(|| {
                let _result = ferrous.convert(&data);
                black_box(&_result);
            })
        });

        // opencc-fmmseg
        let mut opencc_fmmseg = opencc_fmmseg::OpenCC::new();
        group.bench_function("opencc-fmmseg-rayon", |b| {
            b.iter_with_large_drop(|| {
                let _result = opencc_fmmseg.convert(&data, "t2s", false);
                black_box(&_result);
            })
        });
        opencc_fmmseg.set_parallel(false);
        group.bench_function("opencc-fmmseg-no-rayon", |b| {
            b.iter_with_large_drop(|| {
                let _result = opencc_fmmseg.convert(&data, "t2s", false);
                black_box(&_result);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_compare_to_hans, bench_compare_to_tw);
criterion_main!(benches);
// VIBED
