mod common;

use crate::common::*;
use bijective_map::BiMap;
use criterion::*;
use permutation_iterator::Permutor;
use rand::{thread_rng, RngCore};

fn bench_get(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("get");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for load_factor in [0.5, 0.75, 0.8, 0.9] {
        for length in SIZES {
            let mut map = BiMap::with_capacity(length);
            let mut permutor_left =
                Permutor::new_with_u64_key(u64::MAX, rng.next_u64()).into_iter();
            let mut permutor_right =
                Permutor::new_with_u64_key(u64::MAX, rng.next_u64()).into_iter();

            let entire_length = (length as f64 * (load_factor / 0.9)) as usize;
            for _ in 0..entire_length {
                let left = permutor_left.next().unwrap();
                let right = permutor_right.next().unwrap();
                map.insert(left, right);
            }

            group.bench_with_input(
                BenchmarkId::new(format!("get_{}", load_factor), length),
                &length,
                |b, _| {
                    b.iter_batched(
                        || rng.next_u64() % entire_length as u64,
                        |key| map.get_left(&key),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_get);
criterion_main!(benches);
