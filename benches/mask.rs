use adaptivegrain_rs::mask::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn mask_value(c: &mut Criterion) {
    black_box(FLOAT_RANGE.iter().count());
    let ls = black_box(10.0);
    let y = black_box(0.412323);
    c.bench_function("mask value y=0.412", |b| {
        b.iter(|| {
            FLOAT_RANGE.iter().for_each(|&x| {
                black_box(get_mask_value(black_box(x), y, ls));
            });
        })
    });
}

criterion_group!(mask, mask_value);
criterion_main!(mask);
