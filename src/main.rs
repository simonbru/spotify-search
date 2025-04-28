use std::env;
use std::path::Path;

use clap::{crate_version, App, Arg};

mod config;
mod search;
mod web;

fn truncate_chars(value: &str, max: usize) -> String {
    if max < 3 {
        panic!("Can't truncate to fewer than 3 chars.")
    }
    let size = value.chars().count();
    if size <= max {
        return value.to_string();
    }
    let truncated_value: String = value.chars().take(max - 3).collect();
    return format!("{}...", truncated_value.trim_end());
}

fn format_result(collection_name: &str, track_meta: &search::TrackMeta) -> String {
    let artists: Vec<String> = track_meta
        .track
        .artists
        .iter()
        .map(|artist| artist.name.clone())
        .collect();
    let artists_label = match artists.is_empty() {
        true => "-".to_string(),
        false => artists.join(", "),
    };
    return format!(
        // "{collection}: #{pos}\t{track}\t{artists}"
        "{collection}: #{pos}   {track}  |  {artists}",
        collection = truncate_chars(collection_name, 30),
        pos = track_meta.position,
        artists = artists_label,
        track = track_meta.track.name,
    );
}

fn parse_args<'a>() -> clap::ArgMatches<'a> {
    let app = App::new("spotify-search")
        .version(crate_version!())
        .about("Search for tracks in JSON files produced by simonbru/spotify-backup.")
        .arg(
            Arg::with_name("KEYWORDS")
                // TODO: not required anymore
                .required(true)
                .multiple(true)
                .help("Keywords that must all be part of the track's title or artists."),
        )
        .arg(
            Arg::with_name("library_path")
                .short("p")
                .long("path")
                .value_name("LIBRARY_PATH")
                .help("Path of folder containing Spotify backup.")
                .takes_value(true)
                .default_value(config::DEFAULT_LIBRARY_DIR),
        )
        .arg(
            Arg::with_name("web")
                .long("web")
                .help("Show search results on web UI"),
        );
    app.get_matches()
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = parse_args();
    let library_path = Path::new(matches.value_of("library_path").unwrap());
    let search_keywords: Vec<&str> = matches.values_of("KEYWORDS").unwrap().collect();
    let use_web_ui = matches.is_present("web");

    if use_web_ui {
        web::serve_web_ui(library_path, &search_keywords);
    } else {
        let results = search::search(library_path, &search_keywords);
        println!("COLLECTION:   TRACK  |  ARTISTS");
        println!("-------------------------------");
        for result in results {
            let result_line = format_result(&result.collection.name, &result.track);
            println!("{}", result_line);
        }
    }
    Ok(())
}
