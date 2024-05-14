use ahash::HashSet;
use geo::{coord, Coord, Geometry, MultiPolygon, Rect};
use h3o::{
    geom::{ContainmentMode, PolyfillConfig, ToCells},
    CellIndex, Resolution,
};
use std::f64::consts::PI;

/// Default tile size (from MVT spec).
/// Cf. <https://github.com/mapbox/vector-tile-spec/blob/master/2.1/README.md>
const TILE_SIZE: u32 = 4096;

/// Default buffer size.
///
/// Used to render shapes that overlap multiple adjacent tiles.
const BUFFER: u32 = 80;

/// Tile padding.
const PADDING: f64 = BUFFER as f64 * (1. / TILE_SIZE as f64);

/// A tile identifier in a `xy` grid at zoom level `z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TileID {
    x: u32,
    y: u32,
    z: u32,
}

impl TileID {
    /// Initialize a new tile identifier.
    #[must_use]
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    /// Returns the zoom level of the tile.
    #[must_use]
    pub const fn zoom(&self) -> u32 {
        self.z
    }

    /// Returns true if the tile is in the eastern hemisphere.
    #[must_use]
    pub const fn is_eastern(&self) -> bool {
        self.x > ((1 << self.z) / 2)
    }

    /// Returns the bounding box of the tile.
    ///
    /// The bounding box is represented by a set of H3 cells at the requested
    /// resolution.
    #[must_use]
    pub fn bbox(self, resolution: Resolution) -> HashSet<CellIndex> {
        let zoom_level = self.zoom();

        // At zoom level 0, the whole world is covered.
        if zoom_level == 0 {
            return CellIndex::base_cells()
                .flat_map(move |index| index.children(resolution))
                .collect();
        }

        // Select a polyfill resolution that minimize the computation time.
        //
        // It's a worth it to reduce the accuracy (i.e. covering more surface)
        // of the bbox here.
        // Since intersection is based on H3 cells anyway, a slightly bigger
        // bbox won't be slower, and the buffer allow to clip the protruding
        // parts at rendering time.
        //
        let polyfill_res = std::cmp::min(
            resolution,
            match zoom_level {
                0 | 1 => Resolution::Zero,
                2 => Resolution::One,
                3 | 4 => Resolution::Two,
                5 => Resolution::Three,
                6 | 7 => Resolution::Four,
                8 => Resolution::Five,
                9 => Resolution::Six,
                10 | 11 => Resolution::Seven,
                12 => Resolution::Eight,
                13 | 14 => Resolution::Nine,
                // TODO: use a set of finer resolutions.
                _ => Resolution::Ten,
            },
        );

        // Compute the shape of the bounding box.
        // Note that in some cases it can be more complex than a simple rect.
        let bbox = h3o::geom::Geometry::from_degrees(self.compute_bbox())
            .expect("invalid tile bounding box");
        // We use the `Covers` containment mode to ensure 100% coverage.
        let config = PolyfillConfig::new(polyfill_res)
            .containment_mode(ContainmentMode::Covers);
        // Polyfill at the selected resolution and convert to the requested one.
        bbox.to_cells(config)
            .flat_map(move |cell| cell.children(resolution))
            .collect()
    }

    /// Computes the shape of the padded bounding box of this tile.
    ///
    /// In most cases, it's just a rectangle.
    //
    // But in some cases, very large bounding box and/or bounding box wrapping
    // around the world (e.g. crossing the antimeridian), the bounding box is
    // split into smaller components that can be polyfilled independanly and
    // then merged back to obtain the final H3 coverage.
    fn compute_bbox(self) -> Geometry {
        // Compute the padded bounding box of the tile.
        let (x, y, z) = (self.x, self.y, self.z);
        let nw = TileCoord::with_padding(x, y, z, -PADDING);
        let se = TileCoord::with_padding(x + 1, y + 1, z, PADDING);
        let bbox = Rect::new(nw, se);

        // Common case: a trivial bounding box.
        if bbox_is_trivial(&bbox) {
            return bbox.into();
        }

        // Build a multi-part bounding box.
        let mut parts = MultiPolygon::new(Vec::with_capacity(6));
        // Start by clamping the bounding box within the world.
        let Coord { x: min_x, y: min_y } = bbox.min();
        let Coord { x: max_x, y: max_y } = bbox.max();
        let bbox = Rect::new(
            coord! { x: min_x.max(-180.), y: min_y.max(-90.)},
            coord! { x: max_x.min(180.), y: max_y.min(90.)},
        );
        // Split large bbox to avoid tripping the transmeridian heuristic of H3.
        if bbox.width() > 180. {
            let [left, right] = bbox.split_x();
            parts.0.push(left.into());
            parts.0.push(right.into());
        } else {
            parts.0.push(bbox.into());
        }
        // Then add back the clamped parts (correctly reprojected).
        if min_x < -180. {
            let fix = min_x + 360.;
            let clamped = Rect::new((fix, bbox.min().y), (180., bbox.max().y));
            parts.0.push(clamped.into());
        }
        if max_x > 180. {
            let fix = max_x - 360.;
            let clamped = Rect::new((-180., bbox.min().y), (fix, bbox.max().y));
            parts.0.push(clamped.into());
        }
        if min_y < -90. {
            let fix = min_y + 180.;
            let clamped = Rect::new((bbox.min().x, fix), (bbox.max().x, 90.));
            parts.0.push(clamped.into());
        }
        if max_y > 90. {
            let fix = max_y - 180.;
            let clamped = Rect::new((bbox.min().x, -90.), (bbox.max().x, fix));
            parts.0.push(clamped.into());
        }

        parts.into()
    }
}

/// Check if the bounding box is trivial (i.e. can be processed as-is).
///
/// A bounding box is trivial if it's not overly wide (won't trigger H3
/// transmeridian heuristic) and doesn't wrap around the world on any side.
fn bbox_is_trivial(bbox: &Rect) -> bool {
    bbox.min().x >= -180.
        && bbox.max().x <= 180.
        && bbox.min().y >= -90.
        && bbox.max().y <= 90.
        && bbox.width() <= 180.
}

// -----------------------------------------------------------------------------

/// Coordinate in a `xy` grid at zoom level `z`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TileCoord {
    x: f64,
    y: f64,
    z: u32,
}

impl TileCoord {
    /// Converts the EPSG:4326 coordinates into coordinates in grid at zoom `z`.
    #[must_use]
    pub fn new(coord: Coord, z: u32) -> Self {
        let lat = coord.y.to_radians();
        let n = f64::from(1 << z);
        let x = (coord.x + 180.0) / 360.0 * n;
        let y = (1.0 - lat.tan().asinh() / PI) / 2.0 * n;
        Self { x, y, z }
    }

    /// Reprojects this coordinate as centered on the specified tile.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // MVT use integer coordinates.
    pub fn project(self, tile: TileID) -> Coord {
        let center = Self::from(tile);
        Coord {
            x: f64::from(((self.x - center.x) * f64::from(TILE_SIZE)) as i32),
            y: f64::from(((self.y - center.y) * f64::from(TILE_SIZE)) as i32),
        }
    }

    /// Creates a new tile coordinate with from a tile with padding.
    fn with_padding(x: u32, y: u32, z: u32, padding: f64) -> Self {
        Self {
            x: f64::from(x) + padding,
            y: f64::from(y) + padding,
            z,
        }
    }
}

impl From<TileCoord> for Coord {
    fn from(value: TileCoord) -> Self {
        let n = f64::from(1 << value.z);
        let lng = (value.x / n).mul_add(360.0, -180.0);
        let lat = ((1. - 2. * value.y / n) * PI).sinh().atan();
        Self {
            x: lng,
            y: lat.to_degrees(),
        }
    }
}

impl From<TileID> for TileCoord {
    fn from(value: TileID) -> Self {
        Self {
            x: f64::from(value.x),
            y: f64::from(value.y),
            z: value.z,
        }
    }
}

#[cfg(test)]
#[path = "./tile_tests.rs"]
mod tests;
