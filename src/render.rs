use crate::{
    ring_hierarchy::RingHierarchy, tile::TileCoord, RenderingError, TileID,
};
use ahash::HashSet;
use geo::{
    line_string, BooleanOps, BoundingRect, Contains, Coord, Geometry,
    Intersects, LineString, MultiPolygon, Polygon, Rect, Winding,
};
use geozero::{mvt::tile::Layer, ToMvt};
use h3o::{geom::SolventBuilder, CellIndex, LatLng};
use std::{collections::VecDeque, ops::RangeInclusive};

/// Returns every tile ID touched by a given cell index in the specified zoom
/// range.
// TODO: use RangeBounds and return an iterator?
#[must_use]
pub fn tiles_for_cell(
    cell: CellIndex,
    zoom: RangeInclusive<u8>,
) -> HashSet<TileID> {
    let mut tiles = HashSet::default();
    let coord: Coord = LatLng::from(cell).into();
    let tile = TileCoord::from_ll(coord, *zoom.end()).tile_id();
    let boundary = CellBoundary::from(cell);

    // This one is guaranteed to include the cell since it contains its center.
    tiles.insert(tile);

    // Now check if the cell is fully contained in the tile or if it overflows.
    // In the latter case, we need to add the adjacent tiles.
    if !boundary.is_inside(&tile) {
        let mut seen = tile.neighbors().collect::<HashSet<_>>();
        let mut candidates = tile.neighbors().collect::<VecDeque<_>>();
        while let Some(tile) = candidates.pop_front() {
            if boundary.intersects(&tile) {
                tiles.insert(tile);
                candidates
                    .extend(tile.neighbors().filter(|tile| seen.insert(*tile)));
            }
        }
    }

    let mut queue = tiles.iter().copied().collect::<VecDeque<_>>();
    // Ok that we have handled the smaller zoom level, we can bubble up.
    while let Some(tile) = queue.pop_front() {
        // We have reached the requested zoom level.
        if tile.zoom().saturating_sub(1) < *zoom.start() {
            break;
        }
        // We have reached the top level.
        let Some(parent) = tile.parent(tile.zoom().saturating_sub(1)) else {
            break;
        };
        if tiles.insert(parent) {
            queue.push_back(parent);
        }
    }

    tiles
}

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
    let solvent = SolventBuilder::new().build();
    let geometry = solvent
        .dissolve(cells)
        .map_err(RenderingError::InvalidInput)
        .map(|shape| (!shape.0.is_empty()).then_some(shape))?;

    let mut features = Vec::with_capacity(1);
    if let Some(geometry) = geometry {
        let mut geometry = MultiPolygon::new(
            geometry
                .into_iter()
                .map(|mut polygon| {
                    project_polygon_into_grid(&mut polygon, tile_id);
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

        // Clip the resulting geometry using the buffered tile shape.
        //
        // This results in more correct line interpolations, thus preventing
        // distortions and mismatches at the tile edges for shape that overlap
        // several tiles (this tend to become visible at high zoom levels such
        // as 19+).
        let bbox = TileID::buffered_shape().to_polygon();
        geometry = geometry.intersection(&MultiPolygon(vec![bbox]));

        if !geometry.0.is_empty() {
            features.push(
                Geometry::MultiPolygon(geometry)
                    .to_mvt_unscaled()
                    .map_err(RenderingError::Encoding)?,
            );
        }
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

/// Convert the given MVT layer into a Geometry object.
#[expect(clippy::unimplemented, reason = "it's ok, this is for test only")]
#[cfg(test)]
pub fn layer_to_geometry(tile_id: TileID, layer: &Layer) -> Option<Geometry> {
    // No feature, no geometry.
    if layer.features.is_empty() {
        return None;
    }

    let mut geo_writer = geozero::geo_types::GeoWriter::new();
    geozero::mvt::process_geom(&layer.features[0], &mut geo_writer)
        .expect("read MVT geometry");
    geo_writer.take_geometry().map(|mut geometry| {
        match geometry {
            Geometry::Polygon(ref mut polygon) => {
                project_polygon_into_epgs4326(polygon, tile_id);
            }
            Geometry::MultiPolygon(ref mut multipolygon) => {
                for polygon in multipolygon {
                    project_polygon_into_epgs4326(polygon, tile_id);
                }
            }
            // In this context, we only expect polygons or multipolygons.
            _ => unimplemented!("unsupported geometry"),
        }
        geometry
    })
}

// -----------------------------------------------------------------------------

/// Fix shape crossing the antimeridian.
///
/// The shape need to be translated to the east or west, depending on the tile
/// we want to render.
fn fix_transmeridian(tile_id: TileID, ring: &mut LineString<f64>) {
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
fn project_polygon_into_grid(polygon: &mut Polygon, tile_id: TileID) {
    let zoom = tile_id.zoom();

    polygon.exterior_mut(|ring| {
        fix_transmeridian(tile_id, ring);
        for coord in ring.coords_mut() {
            *coord = TileCoord::from_ll(*coord, zoom).project(tile_id);
        }
    });
    polygon.interiors_mut(|interiors| {
        for ring in interiors {
            fix_transmeridian(tile_id, ring);
            for coord in ring.coords_mut() {
                *coord = TileCoord::from_ll(*coord, zoom).project(tile_id);
            }
        }
    });
}

/// Convert a polygon in the given tile to the EPSG:4326 coordinate system.
#[cfg(test)]
fn project_polygon_into_epgs4326(polygon: &mut Polygon, tile_id: TileID) {
    polygon.exterior_mut(|ring| {
        for coord in ring.coords_mut() {
            *coord = TileCoord::from_xy(*coord, tile_id).to_ll();
        }
    });
    polygon.interiors_mut(|interiors| {
        for ring in interiors {
            for coord in ring.coords_mut() {
                *coord = TileCoord::from_xy(*coord, tile_id).to_ll();
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

// -----------------------------------------------------------------------------

// The boundary of a H3 cell.
//
// When the cell cross over the antimeridian, it is represented by two
// projections, one westward and the other eastward.
pub enum CellBoundary {
    Regular(Polygon),
    Transmeridian(Polygon, Polygon),
}

impl CellBoundary {
    /// Check if the cell is fully contained in the tile.
    fn is_inside(&self, tile: &TileID) -> bool {
        fn contains(bbox: &Rect, boundary: &Polygon) -> bool {
            // The polygon of a cell is a glorified ring, there are no holes.
            boundary
                .exterior()
                .coords()
                .all(|coord| bbox.contains(coord))
        }

        let bbox = tile.bbox();
        match self {
            Self::Regular(boundary) => contains(&bbox, boundary),
            Self::Transmeridian(east, west) => {
                if tile.is_eastern() {
                    contains(&bbox, east)
                } else {
                    contains(&bbox, west)
                }
            }
        }
    }

    fn intersects(&self, tile: &TileID) -> bool {
        let bbox = tile.bbox();
        match self {
            Self::Regular(boundary) => boundary.intersects(&bbox),
            Self::Transmeridian(east, west) => {
                if tile.is_eastern() {
                    east.intersects(&bbox)
                } else {
                    west.intersects(&bbox)
                }
            }
        }
    }
}

impl From<CellIndex> for CellBoundary {
    fn from(value: CellIndex) -> Self {
        let boundary = Polygon::from(value);
        let is_transmeridian = boundary
            .exterior()
            .lines()
            .any(|line| (line.start.x - line.end.x).abs() > 180.);

        if is_transmeridian {
            let mut fixed_east = boundary.clone();
            fixed_east.exterior_mut(|exterior| {
                for coord in exterior.coords_mut() {
                    coord.x += f64::from(u8::from(coord.x < 0.)) * 360.;
                }
            });

            let mut fixed_west = boundary;
            fixed_west.exterior_mut(|exterior| {
                for coord in exterior.coords_mut() {
                    coord.x -= f64::from(u8::from(coord.x > 0.)) * 360.;
                }
            });

            Self::Transmeridian(fixed_east, fixed_west)
        } else {
            Self::Regular(boundary)
        }
    }
}

#[cfg(test)]
#[path = "./render_tests.rs"]
mod tests;
