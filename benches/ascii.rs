use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn u8_to_ascii(buffer: &mut[u8]) {
    buffer.iter_mut().for_each(|v| {
        *v = (*v % 93) + 32
    })
}


fn criterion_benchmark(c: &mut Criterion) {
    let mut buf = vec![38_u8; 4096];

    c.bench_function("u8_to_ascii", |b| b.iter(|| u8_to_ascii(black_box(&mut buf))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);