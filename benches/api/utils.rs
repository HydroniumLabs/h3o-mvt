use h3o::{CellIndex, Resolution};
use std::path::PathBuf;

pub fn load_dataset(name: &str, resolution: Resolution) -> Vec<CellIndex> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let filepath = format!("dataset/{name}.cht");
    path.push(filepath);

    let bytes = std::fs::read(path).expect("read test data");
    let mut cells = h3o_zip::decompress(&bytes)
        .map(|res| res.expect("valid test data"))
        .flat_map(|cell| cell.children(Resolution::Ten))
        .map(|cell| cell.parent(resolution).expect("coarser resolution"))
        .collect::<Vec<_>>();
    cells.sort_unstable();
    cells.dedup();

    cells
}
