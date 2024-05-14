use crate::{RenderingError, TileCoord, TileID};
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
) -> Result<Layer, RenderingError> {
    let zoom = tile_id.zoom();
    let mut content =
        cells.to_geom(true).map_err(RenderingError::InvalidInput)?;

    // Reproject the geometry into the XYZ grid coordinate system.
    for polygon in &mut content {
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

    let feature = geo::Geometry::MultiPolygon(content)
        .to_mvt_unscaled()
        .map_err(RenderingError::Encoding)?;

    Ok(Layer {
        extent: None,
        version: 2,
        name,
        features: vec![feature],
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
