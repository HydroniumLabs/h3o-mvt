use crate::{ring_hierarchy::RingHierarchy, RenderingError, TileCoord, TileID};
use geo::{
    bounding_rect::BoundingRect, intersects::Intersects, line_string,
    winding_order::Winding, Geometry, MultiPolygon, Polygon, Rect,
};
use geozero::{mvt::tile::Layer, ToMvt};
use h3o::{geom::ToGeo, CellIndex};

/// Render the given cells into the specified tile.
///
/// # Errors
///
/// All cell indexes must be unique and have the same resolution, otherwise a
/// `RenderingError::InvalidInput` is returned.
///
/// If for some reason the geometry cannot be encoded into an MVT layer, a
/// `RenderingError::Encoding` is returned, carrying the underlying error.
pub fn render(
    tile_id: TileID,
    cells: impl IntoIterator<Item = CellIndex>,
    name: String,
    scratch: bool,
) -> Result<Layer, RenderingError> {
    let geometry = cells
        .to_geom(true)
        .map_err(RenderingError::InvalidInput)
        .map(|shape| (!shape.0.is_empty()).then_some(shape))?;

    let mut features = Vec::with_capacity(1);
    if let Some(geometry) = geometry {
        let mut geometry = MultiPolygon::new(
            geometry
                .into_iter()
                .map(|mut polygon| {
                    reproject_polygon(&mut polygon, tile_id);
                    polygon
                })
                // Ideally we should filter before the map, but it's easier to
                // filter after the reprojection.
                .filter(polygon_is_visible)
                .collect(),
        );

        if scratch {
            geometry = carve_out_from_tile(geometry);
        }

        features.push(
            Geometry::MultiPolygon(geometry)
                .to_mvt_unscaled()
                .map_err(RenderingError::Encoding)?,
        );
    } else if scratch {
        // If there are no shape in scratch mode, we still need to render the
        // tile itself.
        features.push(
            Geometry::Rect(TileID::buffered_shape())
                .to_mvt_unscaled()
                .map_err(RenderingError::Encoding)?,
        );
    }

    Ok(Layer {
        extent: Some(TileID::extent()),
        version: 2,
        name,
        features,
        ..Layer::default()
    })
}

/// Fix shape crossing the antimeridian.
///
/// The shape need to be translated to the east or west, depending on the tile
/// we want to render.
fn fix_transmeridian(tile_id: TileID, ring: &mut geo::LineString<f64>) {
    let is_transmeridian = ring
        .lines()
        .any(|line| (line.start.x - line.end.x).abs() > 180.);

    if is_transmeridian {
        if tile_id.is_eastern() {
            for coord in ring.coords_mut() {
                coord.x += f64::from(u8::from(coord.x < 0.)) * 360.;
            }
        } else {
            for coord in ring.coords_mut() {
                coord.x -= f64::from(u8::from(coord.x > 0.)) * 360.;
            }
        }
    }
}

/// Reprojects the polygon into the tile.
///
/// Convert from EPSG:4326 coordinate to the tile coordinate system, and then
/// project into the relative tile coordinate.
fn reproject_polygon(polygon: &mut Polygon, tile_id: TileID) {
    let zoom = tile_id.zoom();

    polygon.exterior_mut(|ring| {
        fix_transmeridian(tile_id, ring);
        for coord in ring.coords_mut() {
            *coord = TileCoord::new(*coord, zoom).project(tile_id);
        }
    });
    polygon.interiors_mut(|interiors| {
        for ring in interiors {
            fix_transmeridian(tile_id, ring);
            for coord in ring.coords_mut() {
                *coord = TileCoord::new(*coord, zoom).project(tile_id);
            }
        }
    });
}

/// Check if the polygon is visible in the tile.
///
/// Accuracy is not super important here (shapes outsides of the tile won't be
/// rendered anyway but they bloat the generated tile so the goal here is to
/// prune the obvious ones), being fast is more important.
fn polygon_is_visible(polygon: &Polygon) -> bool {
    let tile = TileID::buffered_shape();
    let bbox = polygon.bounding_rect().expect("bounding box");

    tile.intersects(&bbox)
}

fn carve_out_from_tile(shape: MultiPolygon) -> MultiPolygon {
    // Compute the exterior of the carved out shape.
    // If there are no shape, easy: it's the tile shape.
    // If we do have a shape, it may extend further than the tile bbox!
    let bbox = TileID::buffered_shape();
    let boundaries = shape.bounding_rect().map_or(bbox, |enveloppe| {
        let min_x = bbox.min().x.min(enveloppe.min().x - 1.);
        let min_y = bbox.min().y.min(enveloppe.min().y - 1.);
        let max_x = bbox.max().x.max(enveloppe.max().x + 1.);
        let max_y = bbox.max().y.max(enveloppe.max().y + 1.);
        Rect::new((min_x, min_y), (max_x, max_y))
    });
    let exterior = line_string![
        (x: boundaries.min().x, y: boundaries.min().y),
        (x: boundaries.min().x, y: boundaries.max().y),
        (x: boundaries.max().x, y: boundaries.max().y),
        (x: boundaries.max().x, y: boundaries.min().y),
    ];

    // Deconstruct the shapes into rings.
    // Turn them inside out (exteriors become interiors and vice versa), which
    // is express by a change of winding order.
    let mut rings = shape
        .into_iter()
        .flat_map(|polygon| {
            let (mut exterior, mut interiors) = polygon.into_inner();
            exterior.make_ccw_winding();
            for interior in &mut interiors {
                interior.make_cw_winding();
            }
            std::iter::once(exterior).chain(interiors)
        })
        .collect::<Vec<_>>();
    rings.push(exterior);

    // Then rebuild the hierarchy.
    RingHierarchy::new(rings).into()
}

#[cfg(test)]
#[path = "./render_tests.rs"]
mod tests;
