use criterion::{criterion_group, criterion_main};

mod render;
mod utils;

criterion_group!(
    benches,
    render::render_full,
    render::render_many,
    render::render_sparse,
    render::render_few,
    render::render_empty,
);
criterion_main!(benches);
