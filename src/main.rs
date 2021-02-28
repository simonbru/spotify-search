use std::env;
use std::fs;
use std::path::Path;
use std::process;

use serde_json::Value;

mod config;

fn search_in_tracks<'a>(tracks: &'a Value, query: &str) -> Vec<&'a Value> {
    tracks
        .as_array()
        .unwrap()
        .iter()
        .map(|track_meta| &track_meta["track"])
        .filter(|track| match track["name"].as_str() {
            Some(track_name) => track_name.to_lowercase().contains(&query.to_lowercase()),
            None => false,
        })
        .collect()
}

fn format_result(collection_name: &str, track: &Value) -> String {
    let artists: Vec<&str> = track["artists"]
        .as_array()
        .unwrap()
        .iter()
        .map(|artist| artist["name"].as_str().unwrap())
        .filter(|artist_name| artist_name != &"")
        .collect();
    let artists_label = match artists.is_empty() {
        true => "-".to_string(),
        false => artists.join(", ")
    };
    return format!(
        // "{collection}\t{track}\t{artists}"
        "{collection}:   {track}  |  {artists}",
        collection = collection_name,
        artists = artists_label,
        track = track["name"].as_str().unwrap()
    );
}

fn get_args() -> Option<(String, String)> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [executable, search_query, ..] => Some((executable.clone(), search_query.clone())),
        _ => None,
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (_, search_query) = match get_args() {
        Some(args) => args,
        None => {
            println!("Usage: spotify-search KEYWORD");
            process::exit(1);
        }
    };
    let playlist_dir = Path::new(config::LIBRARY_DIR).join("playlists");

    println!("COLLECTION:   TRACK  |  ARTISTS");
    println!("-------------------------------");
    for entry in fs::read_dir(playlist_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir()
            || path.extension().is_none()
            || path.extension().unwrap().to_str() != Some("json")
        {
            continue;
        }
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        let playlist: Value = serde_json::from_str(contents.as_str())?;
        let tracks = search_in_tracks(&playlist["tracks"]["items"], &search_query);
        for track in tracks {
            let result_line = format_result(playlist["name"].as_str().unwrap(), track);
            println!("{}", result_line);
        }
    }
    Ok(())
}
