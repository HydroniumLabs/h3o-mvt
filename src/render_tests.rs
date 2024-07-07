#![allow(clippy::panic)] // Unit tests, this is fine.

use super::*;
use ahash::HashMap;
use float_eq::{assert_float_eq, float_eq};
use geo::{polygon, Geometry, LineString, Polygon};
use geojson::{feature::Id as FeatureId, Feature, FeatureCollection};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn culling_outside() {
    let poly = polygon![
        (x: 100., y: -100.),
        (x: 100., y: -888.),
        (x: 888., y: -888.),
        (x: 888., y: -100.),
    ];
    assert!(!polygon_is_visible(&poly));
}

#[test]
fn culling_inside() {
    let poly = polygon![
        (x: 100., y: 100.),
        (x: 100., y: 888.),
        (x: 888., y: 888.),
        (x: 888., y: 100.),
    ];
    assert!(polygon_is_visible(&poly));
}

#[test]
fn culling_partial() {
    let poly = polygon![
        (x: -100., y: -100.),
        (x: -100., y: 888.),
        (x: 888.,  y: 888.),
        (x: 888.,  y: -100.),
    ];
    assert!(polygon_is_visible(&poly));
}

#[test]
fn culling_around() {
    let poly = polygon![
        (x: -100., y: -100.),
        (x: -100., y: 5000.),
        (x: 5000., y: 5000.),
        (x: 5000., y: -100.),
    ];
    assert!(polygon_is_visible(&poly));
}

#[test]
fn tiles_for_cell_simple() {
    let cell = CellIndex::try_from(0x8a1fb46664e7fff).expect("cell");
    let mut expected = [
        // 9 tiles at zoom 19.
        TileID::new_unchecked(265543, 180337, 19),
        TileID::new_unchecked(265544, 180337, 19),
        TileID::new_unchecked(265545, 180337, 19),
        TileID::new_unchecked(265543, 180338, 19),
        TileID::new_unchecked(265544, 180338, 19),
        TileID::new_unchecked(265545, 180338, 19),
        TileID::new_unchecked(265543, 180339, 19),
        TileID::new_unchecked(265544, 180339, 19),
        TileID::new_unchecked(265545, 180339, 19),
        // 4 tiles at zoom 18.
        TileID::new_unchecked(132771, 90168, 18),
        TileID::new_unchecked(132772, 90168, 18),
        TileID::new_unchecked(132771, 90169, 18),
        TileID::new_unchecked(132772, 90169, 18),
        // 2 tiles at zoom 17 and 16.
        TileID::new_unchecked(66385, 45084, 17),
        TileID::new_unchecked(66386, 45084, 17),
        TileID::new_unchecked(33192, 22542, 16),
        TileID::new_unchecked(33193, 22542, 16),
        // 1 tiles from zoom 15 to 0.
        TileID::new_unchecked(16596, 11271, 15),
        TileID::new_unchecked(8298, 5635, 14),
        TileID::new_unchecked(4149, 2817, 13),
        TileID::new_unchecked(2074, 1408, 12),
        TileID::new_unchecked(1037, 704, 11),
        TileID::new_unchecked(518, 352, 10),
        TileID::new_unchecked(259, 176, 9),
        TileID::new_unchecked(129, 88, 8),
        TileID::new_unchecked(64, 44, 7),
        TileID::new_unchecked(32, 22, 6),
        TileID::new_unchecked(16, 11, 5),
        TileID::new_unchecked(8, 5, 4),
        TileID::new_unchecked(4, 2, 3),
        TileID::new_unchecked(2, 1, 2),
        TileID::new_unchecked(1, 0, 1),
        TileID::new_unchecked(0, 0, 0),
    ]
    .iter()
    .copied()
    .collect::<HashSet<_>>();

    let result = tiles_for_cell(cell, 0..=19);
    assert_eq!(result, expected, "0..=19");

    expected.remove(&TileID::new_unchecked(0, 0, 0));
    let result = tiles_for_cell(cell, 1..=19);
    assert_eq!(result, expected, "1..=19");

    expected.remove(&TileID::new_unchecked(1, 0, 1));
    let result = tiles_for_cell(cell, 2..=19);
    assert_eq!(result, expected, "2..=19");
}

#[test]
#[allow(clippy::decimal_literal_representation)] // Don't want hex here.
fn tiles_for_cell_antimeridian() {
    let cell = CellIndex::try_from(0x8a9b4361e747fff).expect("cell");
    let expected = [
        // 6 tiles at zoom 19
        TileID::new_unchecked(524287, 287107, 19),
        TileID::new_unchecked(0, 287107, 19),
        TileID::new_unchecked(524287, 287108, 19),
        TileID::new_unchecked(0, 287108, 19),
        TileID::new_unchecked(524287, 287109, 19),
        TileID::new_unchecked(0, 287109, 19),
        // 4 tiles at zoom 18
        TileID::new_unchecked(262143, 143553, 18),
        TileID::new_unchecked(0, 143553, 18),
        TileID::new_unchecked(262143, 143554, 18),
        TileID::new_unchecked(0, 143554, 18),
        // 4 tiles at zoom 17
        TileID::new_unchecked(131071, 71776, 17),
        TileID::new_unchecked(0, 71776, 17),
        TileID::new_unchecked(131071, 71777, 17),
        TileID::new_unchecked(0, 71777, 17),
        // 2 tiles the other, one for each side of the antimeridian.
        TileID::new_unchecked(65535, 35888, 16),
        TileID::new_unchecked(0, 35888, 16),
        TileID::new_unchecked(32767, 17944, 15),
        TileID::new_unchecked(0, 17944, 15),
        TileID::new_unchecked(16383, 8972, 14),
        TileID::new_unchecked(0, 8972, 14),
        TileID::new_unchecked(8191, 4486, 13),
        TileID::new_unchecked(0, 4486, 13),
        TileID::new_unchecked(4095, 2243, 12),
        TileID::new_unchecked(0, 2243, 12),
        TileID::new_unchecked(2047, 1121, 11),
        TileID::new_unchecked(0, 1121, 11),
        TileID::new_unchecked(1023, 560, 10),
        TileID::new_unchecked(0, 560, 10),
        TileID::new_unchecked(511, 280, 9),
        TileID::new_unchecked(0, 280, 9),
        TileID::new_unchecked(255, 140, 8),
        TileID::new_unchecked(0, 140, 8),
        TileID::new_unchecked(127, 70, 7),
        TileID::new_unchecked(0, 70, 7),
        TileID::new_unchecked(63, 35, 6),
        TileID::new_unchecked(0, 35, 6),
        TileID::new_unchecked(31, 17, 5),
        TileID::new_unchecked(0, 17, 5),
        TileID::new_unchecked(15, 8, 4),
        TileID::new_unchecked(0, 8, 4),
        TileID::new_unchecked(7, 4, 3),
        TileID::new_unchecked(0, 4, 3),
        TileID::new_unchecked(3, 2, 2),
        TileID::new_unchecked(0, 2, 2),
        TileID::new_unchecked(1, 1, 1),
        TileID::new_unchecked(0, 1, 1),
        // Except zoom 0 because you only have one tile anyway.
        TileID::new_unchecked(0, 0, 0),
    ]
    .iter()
    .copied()
    .collect::<HashSet<_>>();

    let result = tiles_for_cell(cell, 0..=19);
    assert_eq!(result, expected);
}

#[test]
fn render_cells_antimeridian() {
    let cells = vec![
        0x8a9b4361e0effff,
        0x8a9b4361e0c7fff,
        0x8a9b4361e0d7fff,
        0x8a9b4361e72ffff,
        0x8a9b4361e777fff,
        0x8a9b4361e767fff,
        0x8a9b4361e0cffff,
        0x8a9b4361e05ffff,
        0x8a9b4361e70ffff,
        0x8a9b4361e71ffff,
    ]
    .into_iter()
    .map(CellIndex::try_from)
    .collect::<Result<Vec<_>, _>>()
    .expect("valid cell indexes");

    let tiles: HashMap<u8, HashSet<TileID>> = cells
        .iter()
        .flat_map(|&cell| tiles_for_cell(cell, 15..=19).into_iter())
        .fold(HashMap::default(), |mut acc, tile_id| {
            acc.entry(tile_id.zoom()).or_default().insert(tile_id);
            acc
        });

    test_rendering(&cells, &tiles, "antimeridian", false);
    test_rendering(&cells, &tiles, "antimeridian", true);
}

// -----------------------------------------------------------------------------

// Test MVT tile rendering against the golden files.
fn test_rendering(
    cells: &[CellIndex],
    tiles: &HashMap<u8, HashSet<TileID>>,
    testcase: &'static str,
    scratch: bool,
) {
    let basedir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "testdata", testcase]
        .iter()
        .collect();
    // If this variable is set, we update the golden files.
    // If not, we compare the render output against those golden files.
    let update_mode = std::env::var_os("UPDATE_GOLDEN_FILE").is_some();

    for (zoom, tile_ids) in tiles {
        // Render the requested tiles, skipping the empty ones.
        let geometries = render_tiles_to_geometries(cells, tile_ids, scratch);

        // Build the corresponding filename.
        let suffix = if scratch { "hollow" } else { "filled" };
        let filename = PathBuf::from(format!("{zoom}-{suffix}.json"));
        let filepath = [basedir.clone(), filename].iter().collect::<PathBuf>();

        // In update mode, save the result as a GeoJSON FeatureCollection.
        if update_mode {
            return save_golden_file(&filepath, geometries);
        }

        // Otherwise, check the geometries against the correspondin golden file.
        let references = load_golden_file(&filepath);
        for (tile_name, result) in geometries {
            let expected = references[&tile_name].clone();
            assert_geometry_equals(&result, &expected);
        }
    }
}

// Render the specified tiles into geometries, skip empty ones.
fn render_tiles_to_geometries(
    cells: &[CellIndex],
    tile_ids: &HashSet<TileID>,
    scratch: bool,
) -> HashMap<String, Geometry> {
    tile_ids
        .iter()
        .filter_map(|&tile_id| {
            let layer = render(
                tile_id,
                cells.iter().copied(),
                "test".to_owned(),
                scratch,
            )
            .unwrap_or_else(|err| {
                panic!("tile rendering failed for {tile_id:?}: {err}")
            });

            layer_to_geometry(tile_id, &layer).map(|geometry| {
                let ((x, y), z) = (tile_id.xy(), tile_id.zoom());
                let tile_name = format!("{z}/{x}/{y}");

                (tile_name, geometry)
            })
        })
        .collect()
}

// Persists the specified geometries on disk.
fn save_golden_file(filepath: &Path, geometries: HashMap<String, Geometry>) {
    let feature_collection = FeatureCollection {
        bbox: None,
        features: geometries
            .into_iter()
            .map(|(tile_name, geometry)| Feature {
                bbox: None,
                geometry: Some((&geometry).into()),
                id: Some(FeatureId::String(tile_name)),
                properties: None,
                foreign_members: None,
            })
            .collect(),
        foreign_members: None,
    };

    let payload = feature_collection.to_string();
    fs::write(filepath, payload)
        .unwrap_or_else(|err| panic!("writing {}: {err}", filepath.display()));
}

// Load geometries from a golden file.
fn load_golden_file(filepath: &Path) -> HashMap<String, Geometry> {
    let payload = fs::read_to_string(filepath)
        .unwrap_or_else(|err| panic!("reading {}: {err}", filepath.display()));
    let feature_collection = payload
        .parse::<FeatureCollection>()
        .unwrap_or_else(|err| panic!("parsing {}: {err}", filepath.display()));
    feature_collection
        .features
        .into_iter()
        .map(|feature| {
            let Some(FeatureId::String(tile_name)) = feature.id else {
                panic!(
                    "invalid or missing feature ID in {}",
                    filepath.display()
                )
            };

            let geometry: Geometry = feature
                .geometry
                .unwrap_or_else(|| panic!("missing geometry in {tile_name}"))
                .try_into()
                .unwrap_or_else(|err| {
                    panic!("invalid geometry in {tile_name}: {err}")
                });

            (tile_name, geometry)
        })
        .collect()
}

fn assert_geometry_equals(lhs: &Geometry, rhs: &Geometry) {
    match (lhs, rhs) {
        (Geometry::MultiPolygon(lhs), Geometry::MultiPolygon(rhs)) => {
            assert_eq!(lhs.0.len(), rhs.0.len(), "polygon count");
            for (lhs, rhs) in lhs.0.iter().zip(rhs.0.iter()) {
                assert_polygon_equals(lhs, rhs);
            }
        }
        (Geometry::Polygon(lhs), Geometry::Polygon(rhs)) => {
            assert_polygon_equals(lhs, rhs);
        }
        _ => panic!("unexpected geometry types"),
    }
}

fn assert_polygon_equals(lhs: &Polygon, rhs: &Polygon) {
    assert_linestring_equals(lhs.exterior(), rhs.exterior());
    assert_eq!(lhs.interiors().len(), rhs.interiors().len(), "hole count");
    // TODO: be order-agnostic?
    for (lhs, rhs) in lhs.interiors().iter().zip(rhs.interiors().iter()) {
        assert_linestring_equals(lhs, rhs);
    }
}

/// Assert that two `LineString` are equivalent.
///
/// `LineString` are equivalent if they contains the same point in the same order
/// (but they don't necessarily start at the same point).
fn assert_linestring_equals(lhs: &LineString, rhs: &LineString) {
    // Based on https://gis.stackexchange.com/a/8674
    const EPSILON: f64 = 1e-5;

    let lhs_is_closed = lhs.is_closed();
    let mut lhs = lhs.coords().collect::<Vec<_>>();
    if lhs_is_closed {
        lhs.pop(); // Remove the duplicated coord that close the ring.
    }

    let rhs_is_closed = rhs.is_closed();
    let mut rhs = rhs.coords().collect::<Vec<_>>();
    if rhs_is_closed {
        rhs.pop(); // Remove the duplicated coord that close the ring
    }

    assert_eq!(lhs.len(), rhs.len(), "linestring size mismatch");
    let offset = rhs
        .iter()
        .position(|&coord| {
            float_eq!(coord.x, lhs[0].x, abs <= EPSILON)
                && float_eq!(coord.y, lhs[0].y, abs <= EPSILON)
        })
        .expect("linestring are different");
    for (i, lhs) in lhs.iter().enumerate() {
        let j = (i + offset) % rhs.len();

        assert_float_eq!(lhs.x, rhs[j].x, abs <= EPSILON);
        assert_float_eq!(lhs.y, rhs[j].y, abs <= EPSILON);
    }
}
