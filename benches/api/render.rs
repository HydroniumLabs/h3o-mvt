use super::utils::load_dataset;
use criterion::{BatchSize, Criterion};
use h3o::{CellIndex, Resolution};
use h3o_mvt::TileID;
use std::hint::black_box;

pub fn render_full(c: &mut Criterion) {
    let cells = vec![
        0x8a415cb4c647fff,
        0x8a415cb48907fff,
        0x8a415cb489a7fff,
        0x8a415cb4c66ffff,
        0x8a415cb4c64ffff,
        0x8a415cb48927fff,
        0x8a415cb4c65ffff,
        0x8a415cb48917fff,
        0x8a415cb48937fff,
        0x8a415cb489affff,
    ]
    .into_iter()
    .map(CellIndex::try_from)
    .collect::<Result<Vec<_>, _>>()
    .expect("valid cells");
    let tile = TileID::new(104067, 57709, 17).expect("valid tile id");

    bench_render(c, "Render/Full", cells, tile);
}

pub fn render_many(c: &mut Criterion) {
    let cells = load_dataset("many", Resolution::Ten);
    let tile = TileID::new(1626, 901, 11).expect("valid tile id");

    bench_render(c, "Render/Many", cells, tile);
}

pub fn render_sparse(c: &mut Criterion) {
    let cells = load_dataset("sparse", Resolution::Eight);
    let tile = TileID::new(203, 113, 8).expect("valid tile id");

    bench_render(c, "Render/Sparse", cells, tile);
}

pub fn render_few(c: &mut Criterion) {
    let cells = vec![
        0x8a65b508518ffff,
        0x8a65b5085c57fff,
        0x8a65b50858dffff,
        0x8a65b519b66ffff,
        0x8a65b5083457fff,
        0x8a65b50855a7fff,
        0x8a65b50b434ffff,
        0x8a65b50b4757fff,
        0x8a65b5085ce7fff,
        0x8a65b50b08c7fff,
        0x8a65b50b598ffff,
        0x8a65b50b4267fff,
        0x8a65b50834e7fff,
        0x8a65b50851affff,
        0x8a65b50b08f7fff,
        0x8a65b50b5cb7fff,
    ]
    .into_iter()
    .map(CellIndex::try_from)
    .collect::<Result<Vec<_>, _>>()
    .expect("valid cells");
    let tile = TileID::new(3266, 1926, 12).expect("valid tile id");

    bench_render(c, "Render/Few", cells, tile);
}

pub fn render_empty(c: &mut Criterion) {
    let cells = vec![];
    let tile = TileID::new(518, 352, 10).expect("valid tile id");

    bench_render(c, "Render/Empty", cells, tile);
}

fn bench_render(
    c: &mut Criterion,
    name: &str,
    cells: Vec<CellIndex>,
    tile: TileID,
) {
    let mut group = c.benchmark_group(name);

    group.bench_function("Normal", |b| {
        b.iter_batched(
            || cells.clone(),
            |cells| {
                h3o_mvt::render(
                    black_box(tile),
                    black_box(cells),
                    "empty".to_owned(),
                    false,
                )
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("Carved", |b| {
        b.iter_batched(
            || cells.clone(),
            |cells| {
                h3o_mvt::render(
                    black_box(tile),
                    black_box(cells),
                    "empty".to_owned(),
                    true,
                )
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}
