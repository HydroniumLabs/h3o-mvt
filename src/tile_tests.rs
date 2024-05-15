#![allow(clippy::panic)] // Unit tests

use super::*;
use float_eq::assert_float_eq;
use geo::polygon;

macro_rules! cells {
        ($($x: expr),* $(,)?) => {{
            vec![
                $(CellIndex::try_from($x).expect("valid cell"),)*
            ].into_iter().collect::<HashSet<_>>()
        }}
    }

#[test]
fn bbox_z0() {
    let tile = TileID::new(0, 0, 0);
    let expected = CellIndex::base_cells().collect::<HashSet<_>>();

    let result = tile.cells(Resolution::Zero);

    assert_eq!(result, expected);
}

#[test]
fn regular_bbox() {
    let tile = TileID::new(265544, 180338, 19);
    let expected = cells![0x8a1fb46664e7fff];

    let result = tile.cells(Resolution::Ten);

    assert_eq!(result, expected);
}

#[test]
fn bbox_transmeridian_right() {
    let tile = TileID::new(0, 71776, 17);
    let expected = cells![
        0x8a9b4361e607fff,
        0x8a9b4361e62ffff,
        0x8a9b4361e677fff,
        0x8a9b4361e627fff,
        0x8a9b4361e667fff,
        0x8a9b4361e70ffff,
        0x8a9b4361e75ffff,
        0x8a9b4361e747fff,
        0x8a9b4361e71ffff,
        0x8a9b4361e60ffff,
        0x8a9b4361e657fff,
        0x8a9b4361e757fff,
        0x8a9b4361e61ffff,
        0x8a9b4361e647fff,
    ];

    let result = tile.cells(Resolution::Ten);

    assert_eq!(result, expected);
}

#[test]
fn bbox_transmeridian_left() {
    #![allow(clippy::decimal_literal_representation)] // False positive.
    let tile = TileID::new(131071, 71776, 17);
    let expected = cells![
        0x8a9b4361e75ffff,
        0x8a9b4361e74ffff,
        0x8a9b4361e29ffff,
        0x8a9b4361e287fff,
        0x8a9b4361e747fff,
        0x8a9b4361e76ffff,
        0x8a9b4361e297fff,
        0x8a9b4361e28ffff,
        0x8a9b4361e66ffff,
        0x8a9b4361e2b7fff,
        0x8a9b4361e2a7fff,
        0x8a9b4361e667fff,
        0x8a9b4361e647fff,
        0x8a9b4361a927fff,
    ];

    let result = tile.cells(Resolution::Ten);

    assert_eq!(result, expected);
}

// Zoom level 1 is tricky because it features:
// - Wide bounding boxes
// - Bounding boxex crossing the antimeridian
//
// Hence the following tests.
#[test]
fn bbox_z1_nw() {
    let tile = TileID::new(0, 0, 1);
    let result = tile.compute_bbox();
    let expected = Geometry::MultiPolygon(MultiPolygon(vec![
        // Main bbox, left part.
        polygon![
            (x: -180.0,     y: -3.51342),
            (x:  -88.24218, y: -3.51342),
            (x:  -88.24218, y: 85.34532),
            (x: -180.0,     y: 85.34532),
        ],
        // Main bbox, right part.
        polygon![
            (x: -88.24218, y: -3.51342),
            (x:  3.515625, y: -3.51342),
            (x:  3.515625, y: 85.34532),
            (x: -88.24218, y: 85.34532),
        ],
        // Clamped left part, reprojected to the right.
        polygon![
            (x: 176.48437, y: -3.51342),
            (x: 180.0,     y: -3.51342),
            (x: 180.0,     y: 85.34532),
            (x: 176.48437, y: 85.34532),
        ],
    ]));
    assert_bbox_equals(&result, &expected);
}

#[test]
fn bbox_z1_ne() {
    let tile = TileID::new(1, 0, 1);
    let result = tile.compute_bbox();
    let expected = Geometry::MultiPolygon(MultiPolygon(vec![
        // Main bbox, left part.
        polygon![
            (x: -3.51562, y: -3.51342),
            (x: 88.24218, y: -3.51342),
            (x: 88.24218, y: 85.34532),
            (x: -3.51562, y: 85.34532),
        ],
        // Main bbox, right part.
        polygon![
            (x:  88.24218, y: -3.51342),
            (x: 180.0,     y: -3.51342),
            (x: 180.0,     y: 85.34532),
            (x:  88.24218, y: 85.34532),
        ],
        // Clamped right part, reprojected to the left.
        polygon![
            (x: -180.0,     y: -3.51342),
            (x: -176.48437, y: -3.51342),
            (x: -176.48437, y: 85.34532),
            (x: -180.0,     y: 85.34532),
        ],
    ]));
    assert_bbox_equals(&result, &expected);
}

#[test]
fn bbox_z1_sw() {
    let tile = TileID::new(0, 1, 1);
    let result = tile.compute_bbox();
    let expected = Geometry::MultiPolygon(MultiPolygon(vec![
        // Main bbox, left part.
        polygon![
            (x: -180.0,     y: -85.34532),
            (x:  -88.24218, y: -85.34532),
            (x:  -88.24218, y:   3.51342),
            (x: -180.0,     y:   3.51342),
        ],
        // Main bbox, right part.
        polygon![
            (x: -88.2421875, y: -85.34532),
            (x:   3.515625,  y: -85.34532),
            (x:   3.515625,  y:   3.51342),
            (x: -88.2421875, y:   3.51342),
        ],
        // Clamped left part, reprojected to the right.
        polygon![
            (x: 176.48437, y: -85.34532),
            (x: 180.0,     y: -85.34532),
            (x: 180.0,     y:   3.51342),
            (x: 176.48437, y:   3.51342),
        ],
    ]));
    assert_bbox_equals(&result, &expected);
}

#[test]
fn bbox_z1_se() {
    let tile = TileID::new(1, 1, 1);
    let result = tile.compute_bbox();
    let expected = Geometry::MultiPolygon(MultiPolygon(vec![
        // Main bbox, left part.
        polygon![
            (x: -3.51562, y: -85.34532),
            (x: 88.24218, y: -85.34532),
            (x: 88.24218, y:   3.51342),
            (x: -3.51562, y:   3.51342),
        ],
        // Main bbox, right part.
        polygon![
            (x:  88.24218, y: -85.34532),
            (x: 180.0,     y: -85.34532),
            (x: 180.0,     y:   3.51342),
            (x:  88.24218, y:   3.51342),
        ],
        // Clamped right part, reprojected to the left.
        polygon![
            (x: -180.0,     y: -85.34532),
            (x: -176.48437, y: -85.34532),
            (x: -176.48437, y:   3.51342),
            (x: -180.0,     y:   3.51342),
        ],
    ]));
    assert_bbox_equals(&result, &expected);
}

fn assert_bbox_equals(lhs: &Geometry, rhs: &Geometry) {
    // Based on https://gis.stackexchange.com/a/8674
    const EPSILON: f64 = 1e-5;

    match (lhs, rhs) {
        (Geometry::MultiPolygon(lhs), Geometry::MultiPolygon(rhs)) => {
            assert_eq!(lhs.0.len(), rhs.0.len(), "bbox pieces count");
            for (lhs, rhs) in lhs.0.iter().zip(rhs.0.iter()) {
                assert!(
                    lhs.interiors().is_empty() && rhs.interiors().is_empty(),
                    "hole in bbox"
                );
                let lhs = lhs.exterior();
                let rhs = rhs.exterior();
                for (lhs, rhs) in lhs.coords().zip(rhs.coords()) {
                    assert_float_eq!(lhs.x, rhs.x, abs <= EPSILON);
                    assert_float_eq!(lhs.y, rhs.y, abs <= EPSILON);
                }
            }
        }
        _ => panic!("unexpected bbox types"),
    }
}
