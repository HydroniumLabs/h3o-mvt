use ahash::{HashMap, HashSet};
use axum::{
    Router,
    extract::{Path, State},
    http::Method,
    response::IntoResponse,
    routing::get,
};
use clap::Parser;
use geozero::mvt::Message as _;
use h3o::{CellIndex, Resolution};
use h3o_mvt::TileID;
use std::sync::OnceLock;
use tower_http::cors::{Any, CorsLayer};

/// A vector tile server for H3 dataset.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Address to which the server should bind.
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
    // Port to use.
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
    // Path to the H3 dataset.
    #[arg(short, long, default_value = "data.cht")]
    dataset: std::path::PathBuf,
    // Scratch off the shape on the map.
    #[arg(long, default_value_t = false)]
    scratch: bool,
}

#[derive(Clone, Copy)]
struct Config {
    scratch: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config {
        scratch: args.scratch,
    };
    load_dataset(&args.dataset);

    let app = Router::new()
        .route("/:z/:x/:y", get(handler))
        .with_state(config)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any),
        );

    let address = format!("{}:{}", args.address, args.port);
    let listener = tokio::net::TcpListener::bind(&address).await.expect("bind");
    axum::serve(listener, app).await.expect("start the server");
}

async fn handler(
    State(state): State<Config>,
    Path((z, x, y)): Path<(u8, u32, u32)>,
) -> impl IntoResponse {
    let tile_id = TileID::new(x, y, z).expect("valid tile ID");
    let resolution = match tile_id.zoom() {
        0..=2 => Resolution::Four,
        3 | 4 => Resolution::Five,
        5 => Resolution::Six,
        6 | 7 => Resolution::Seven,
        8..=9 => Resolution::Eight,
        10 => Resolution::Nine,
        _ => Resolution::Ten,
    };
    let data = get_data(resolution);

    // At zoom level 0, the whole world is covered.
    let content = if tile_id.zoom() == 0 {
        data.clone()
    } else {
        let bbox = tile_id.cells(resolution);
        let bbox = bbox.into_iter().collect::<HashSet<_>>();
        data & &bbox
    };
    // The name here must match the `source-layer` in `viewers.html`.
    let layer =
        h3o_mvt::render(tile_id, content, "h3".to_owned(), state.scratch)
            .expect("rendered MVT layer");
    let tile = geozero::mvt::Tile {
        layers: vec![layer],
    };
    tile.encode_to_vec()
}

// -----------------------------------------------------------------------------

static DATA: OnceLock<HashMap<Resolution, HashSet<CellIndex>>> =
    OnceLock::new();

fn load_dataset(path: &std::path::Path) {
    let bytes = std::fs::read(path).expect("read dataset");
    let indexes = h3o_zip::decompress(bytes.as_slice())
        .flat_map(|index| {
            index.expect("corrupted data").children(Resolution::Ten)
        })
        .collect::<HashSet<_>>();

    // Precompute data shape for the supported resolutions.
    let mut data = Resolution::range(Resolution::Four, Resolution::Nine)
        .map(|res| {
            let cells = indexes
                .iter()
                .copied()
                .map(|cell| cell.parent(res).expect("supported resolution"))
                .collect::<HashSet<_>>();
            (res, cells)
        })
        .collect::<HashMap<Resolution, HashSet<CellIndex>>>();
    data.insert(Resolution::Ten, indexes);

    DATA.set(data).expect("set pre-computed data");
}

// Get the dataset scaled at the requested resolution.
fn get_data(resolution: Resolution) -> &'static HashSet<CellIndex> {
    &DATA.get().expect("requested resolution not pre-computed")[&resolution]
}
