use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lib::performance_indicators::*;
use rand::prelude::*;


pub fn benchmark_min(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("min");
    for vec_size in [1, 10, 100, 1000, 10000].iter() {
        let vals: Vec<f64> = (0..*vec_size).map(|_| rng.gen_range(0.0..100.0)).collect();
        group.bench_with_input(BenchmarkId::new("Array of values", vec_size), &vals, |b, vals| b.iter(|| min(black_box(&vals))));

    }
}


pub fn benchmark_max(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("max");
    for vec_size in [1, 10, 100, 1000, 10000].iter() {
        let vals: Vec<f64> = (0..*vec_size).map(|_| rng.gen_range(0.0..100.0)).collect();
        group.bench_with_input(BenchmarkId::new("Array of values", vec_size), &vals, |b, vals| b.iter(|| max(black_box(&vals))));

    }
}

pub fn benchmark_price_diff(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("price_diff");
    for vec_size in [1, 10, 100, 1000, 10000].iter() {
        let vals: Vec<f64> = (0..*vec_size).map(|_| rng.gen_range(0.0..100.0)).collect();
        group.bench_with_input(BenchmarkId::new("Array of values", vec_size), &vals, |b, vals| b.iter(|| price_diff(black_box(&vals))));

    }
}

pub fn benchmark_n_window_sma(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("min");
    for vec_size in [1, 10, 100, 1000, 10000].iter() {
        let vals: Vec<f64> = (0..*vec_size).map(|_| rng.gen_range(0.0..100.0)).collect();
        group.bench_with_input(BenchmarkId::new("Array of values", vec_size), &vals, |b, vals| b.iter(|| n_window_sma(10, black_box(&vals))));

    }
}
criterion_group!(performance_indicators, benchmark_min, benchmark_max, benchmark_price_diff, benchmark_n_window_sma);
criterion_main!(performance_indicators);