use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(RustEmbed)]
#[folder = "src/assets/web"]
struct Asset;

fn guess_mime_type(filename: &str) -> Option<&str> {
    match filename {
        _ if filename.ends_with(".css") => Some("text/css"),
        _ if filename.ends_with(".js") => Some("application/javascript"),
        _ if filename.ends_with(".html") => Some("text/html"),
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

#[derive(Deserialize)]
struct SearchQueryParams {
    q: String,
}

#[derive(Serialize)]
struct SearchResults {
    hello: String,
}

fn search_view(params: SearchQueryParams) -> warp::reply::Json {
    let response = SearchResults { hello: params.q };
    warp::reply::json(&response)
}

#[tokio::main]
pub async fn serve_web_ui() {
    // pretty_env_logger::init();

    let search = warp::path!("api" / "search")
        .and(warp::get())
        .and(warp::query())
        .map(search_view);

    // let readme = warp::get()
    //     .and(warp::path::end())
    //     .and(warp::fs::file("./README.md"));

    // dir already requires GET...
    // let examples = warp::path("ex").and(warp::fs::dir("./examples/"));

    let assets = warp::path("static")
        .and(warp::path::tail())
        .and(warp::get())
        .and_then(assets_view);
    let assets_index = warp::path::end()
        .and(warp::get())
        .and_then(assets_index_view);

    let routes = assets_index.or(assets).or(search);

    // let routes = routes.with(warp::log("blabla"));

    println!("Listening on http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
