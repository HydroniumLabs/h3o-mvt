use super::*;
use geo::polygon;

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
