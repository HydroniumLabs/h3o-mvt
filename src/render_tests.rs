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
