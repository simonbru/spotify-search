use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use warp::Filter;

use super::search;

#[derive(RustEmbed)]
#[folder = "src/assets/web"]
struct Asset;

const SEARCH_PAGE_SIZE: usize = 200;
const FALLBACK_ALBUM_THUMBNAIL_URL: &str = "/static/fallback-cover.svg";

fn guess_mime_type(filename: &str) -> Option<&str> {
    match filename {
        _ if filename.ends_with(".css") => Some("text/css"),
        _ if filename.ends_with(".js") => Some("application/javascript"),
        _ if filename.ends_with(".html") => Some("text/html"),
        _ if filename.ends_with(".svg") => Some("image/svg+xml"),
        _ => None,
    }
}

fn serve_asset(path: &str) -> Result<impl warp::Reply, warp::Rejection> {
    let content = Asset::get(&path).ok_or_else(warp::reject::not_found)?;
    let mut response = warp::reply::Response::new(content.into());

    if let Some(mime_type) = path
        .rsplit("/")
        .next()
        .and_then(|filename| guess_mime_type(filename))
    {
        response.headers_mut().insert(
            "content-type",
            warp::http::HeaderValue::from_str(mime_type).unwrap(),
        );
    }
    Ok(response)
}

async fn assets_view(path: warp::path::Tail) -> Result<impl warp::Reply, warp::Rejection> {
    serve_asset(path.as_str())
}

async fn assets_index_view() -> Result<impl warp::Reply, warp::Rejection> {
    serve_asset("index.html")
}

#[derive(Serialize)]
struct ImportMap {
    imports: BTreeMap<String, String>,
}

fn import_map_view() -> warp::reply::Json {
    let mut imports = BTreeMap::new();
    imports.insert(
        "vue".to_string(),
        if cfg!(debug_assertions) {
            "/static/vendor/vue@3.2.31/vue.esm-browser.js".to_string()
        } else {
            "/static/vendor/vue@3.2.31/vue.esm-browser.prod.js".to_string()
        },
    );
    warp::reply::json(&ImportMap { imports })
}

#[derive(Deserialize)]
struct SearchQueryParams {
    q: String,
}

#[derive(Serialize)]
struct SearchResponseItem {
    title: String,
    artists: Vec<String>,
    uri: String,
    collection: String,
    thumbnail_url: String,
}

#[derive(Serialize)]
struct SearchResponse {
    items: Vec<SearchResponseItem>,
    total: usize,
}

fn search_view(library_path: &Path, params: SearchQueryParams) -> warp::reply::Json {
    let keywords: Vec<&str> = params.q.split_whitespace().collect();
    let results = search::search(&library_path, &keywords);
    let response = SearchResponse {
        total: results.len(),
        items: results
            .into_iter()
            .take(SEARCH_PAGE_SIZE)
            .map(|result| SearchResponseItem {
                title: result.track.track.name,
                artists: result
                    .track
                    .track
                    .artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .collect(),
                uri: result.track.track.uri,
                collection: result.collection,
                thumbnail_url: result
                    .track
                    .track
                    .album
                    .images
                    .into_iter()
                    .min_by_key(|item| item.height)
                    .map(|item| item.url)
                    .unwrap_or(FALLBACK_ALBUM_THUMBNAIL_URL.to_owned()),
            })
            .collect(),
    };
    warp::reply::json(&response)
}

#[tokio::main]
pub async fn serve_web_ui(library_path: &Path) {
    let search_view_closure = {
        let library_path = library_path.to_owned();
        move |params| search_view(&library_path, params)
    };
    let search = warp::path!("api" / "search")
        .and(warp::get())
        .and(warp::query())
        .map(search_view_closure);

    let import_map = warp::path!("import-map.importmap")
        .and(warp::get())
        .map(import_map_view)
        .map(|reply| warp::reply::with_header(reply, "content-type", "application/importmap+json"));

    let assets = warp::path("static")
        .and(warp::path::tail())
        .and(warp::get())
        .and_then(assets_view);
    let assets_index = warp::path::end()
        .and(warp::get())
        .and_then(assets_index_view);

    let routes = assets_index.or(assets).or(search).or(import_map);

    println!("Listening on http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
