use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_render_loop(c: &mut Criterion) {
    let width = 64;
    let height = 64;
    let map_matrix: Vec<u8> = vec![0x00; width * height];
    let start_x = 0;
    let start_y = 0;
    let view_w = 16;
    let view_h = 8;

    c.bench_function("render_loop_get", |b| {
        b.iter(|| {
            let mut count = 0;
            for row in start_y..(start_y + view_h) {
                for col in start_x..(start_x + view_w) {
                    let idx = row * width + col;
                    let cell_byte = map_matrix.get(idx).copied().unwrap_or(0x00);
                    count += cell_byte as usize;
                }
            }
            black_box(count);
        })
    });

    c.bench_function("render_loop_slice", |b| {
        b.iter(|| {
            let mut count = 0;
            for row in start_y..(start_y + view_h) {
                // direct slicing avoids bounds checks per element
                let start_idx = row * width + start_x;
                let end_idx = start_idx + view_w;
                if end_idx <= map_matrix.len() {
                    for &cell_byte in &map_matrix[start_idx..end_idx] {
                        count += cell_byte as usize;
                    }
                }
            }
            black_box(count);
        })
    });
}

criterion_group!(benches, bench_render_loop);
criterion_main!(benches);
