use std::env;
use std::fs;
use std::path::Path;

#[macro_use]
extern crate clap;
use clap::{App, Arg};
use serde::{Deserialize, Deserializer};
use unidecode::unidecode;

mod config;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
struct Artist {
    name: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
struct Track {
    uri: String,
    name: String,

    #[serde(deserialize_with = "exclude_invalid_artists")]
    artists: Vec<Artist>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
struct TrackMeta {
    added_at: String,
    track: Track,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
struct TracksPage {
    #[serde(deserialize_with = "exclude_null_tracks")]
    items: Vec<TrackMeta>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
struct Playlist {
    name: String,
    tracks: TracksPage,
}

type LibraryTracks = Vec<TrackMeta>;

/// When a track has no artist, its list of artist contains a single artist with empty values.
fn exclude_invalid_artists<'de, D>(deserializer: D) -> Result<Vec<Artist>, D::Error>
where
    D: Deserializer<'de>,
{
    let artists: Vec<Artist> = Deserialize::deserialize(deserializer)?;
    let filtered_artists = artists
        .iter()
        .filter(|artist| artist.name != "")
        .cloned()
        .collect();
    return Ok(filtered_artists);
}

/// Some playlist contain items with dummy metadata and "track: null"
fn exclude_null_tracks<'de, D>(deserializer: D) -> Result<Vec<TrackMeta>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
    struct NullableTrackMeta {
        added_at: String,
        track: Option<Track>,
    }

    let items: Vec<NullableTrackMeta> = Deserialize::deserialize(deserializer)?;
    let track_metas = items
        .iter()
        .filter(|nullable_track_meta| nullable_track_meta.track.is_some())
        .cloned()
        .map(|track_meta| TrackMeta {
            added_at: track_meta.added_at,
            track: track_meta.track.unwrap(),
        })
        .collect();
    return Ok(track_metas);
}

fn search_in_tracks<'a>(tracks: &'a Vec<TrackMeta>, keywords: &Vec<&str>) -> Vec<&'a TrackMeta> {
    tracks
        .iter()
        .filter(|track_meta| match_track(&track_meta.track, keywords))
        .collect()
}

fn match_track(track: &Track, keywords: &Vec<&str>) -> bool {
    let track_name = normalize_keyword(&track.name);
    let artist_names: Vec<String> = track
        .artists
        .iter()
        .map(|artist| normalize_keyword(&artist.name))
        .collect();

    let contains_keyword = |raw_keyword: &str| {
        let keyword = normalize_keyword(raw_keyword);
        if track_name.contains(&keyword) {
            return true;
        }
        for artist_name in &artist_names {
            if artist_name.contains(&keyword) {
                return true;
            }
        }
        false
    };

    keywords.iter().all(|keyword| contains_keyword(keyword))
}

fn normalize_keyword(value: &str) -> String {
    let mut value = unidecode(value)
        .to_lowercase()
        .replace("$", "s")
        .replace("&", "and");
    // Remove chars that often appear inside words (e.g. "P.O.W.E.R")
    for skip_char in "-*.:'".chars() {
        value = value.replace(skip_char, "");
    }
    // Normalize spaces
    let words: Vec<&str> = value.split_whitespace().collect();
    words.join(" ")
}

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

fn format_result(collection_name: &str, track_meta: &TrackMeta) -> String {
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
        // "{collection}\t{track}\t{artists}"
        "{collection}:   {track}  |  {artists}",
        collection = truncate_chars(collection_name, 30),
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
        );
    app.get_matches()
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = parse_args();
    let library_path = Path::new(matches.value_of("library_path").unwrap());
    let search_keywords: Vec<&str> = matches.values_of("KEYWORDS").unwrap().collect();

    println!("COLLECTION:   TRACK  |  ARTISTS");
    println!("-------------------------------");
    // Search tracks in library
    let search_in_library = || {
        let path = Path::new(library_path).join("tracks.json");

        let contents = fs::read_to_string(&path).expect(&format!("Could not read {:?}", path));

        let tracks: LibraryTracks = match serde_json::from_str(&contents) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Could not parse {:?}: {}", path.file_name().unwrap(), err);
                return;
            }
        };
        let tracks = search_in_tracks(&tracks, &search_keywords);
        for track in tracks {
            let result_line = format_result("Library", track);
            println!("{}", result_line);
        }
    };
    search_in_library();

    // Search tracks in playlists
    let playlist_dir = Path::new(library_path).join("playlists");
    for entry in fs::read_dir(playlist_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir()
            || path.extension().is_none()
            || path.extension().unwrap().to_str() != Some("json")
        {
            continue;
        }
        // println!("Parsing {:?}", path);
        let contents = fs::read_to_string(&path).expect(&format!("Could not read {:?}", path));

        let playlist: Playlist = match serde_json::from_str(&contents) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Could not parse {:?}: {}", path.file_name().unwrap(), err);
                continue;
            }
        };
        let tracks = search_in_tracks(&playlist.tracks.items, &search_keywords);
        for track in tracks {
            let result_line = format_result(&playlist.name, track);
            println!("{}", result_line);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_playlist() {
        let test_playlist_str = r#"
            {
                "name": "my_playlist",
                "tracks": {
                    "items": [
                        {
                            "added_at": "2010-08-23T10:33:01Z",
                            "track": {
                                "uri": "spotify:track:asdfasdf",
                                "name": "My track",
                                "artists": [
                                    {
                                        "name": "My artist"
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        "#;
        let test_playlist: Playlist = serde_json::from_str(&test_playlist_str).unwrap();
        let expected_playlist = Playlist {
            name: "my_playlist".to_string(),
            tracks: TracksPage {
                items: vec![TrackMeta {
                    added_at: "2010-08-23T10:33:01Z".to_string(),
                    track: Track {
                        uri: "spotify:track:asdfasdf".to_string(),
                        name: "My track".to_string(),
                        artists: vec![Artist {
                            name: "My artist".to_string(),
                        }],
                    },
                }],
            },
        };
        assert_eq!(test_playlist, expected_playlist)
    }

    #[test]
    fn parse_track_without_artists() {
        let test_track_str = r#"
            {
                "uri": "spotify:track:asdfasdf",
                "name": "My track",
                "artists": [
                    {
                        "name": ""
                    }
                ]
            }
        "#;
        let test_track: Track = serde_json::from_str(&test_track_str).unwrap();
        assert_eq!(test_track.artists.len(), 0)
    }

    #[test]
    fn parse_tracks_page_exclude_null_tracks() {
        let test_tracks_page_str = r#"
            {
                "items": [
                    {
                        "added_at": "1970-01-01T00:00:00Z",
                        "track": null
                    },
                    {
                        "added_at": "2010-08-23T10:33:01Z",
                        "track": {
                            "uri": "spotify:track:asdfasdf",
                            "name": "My track",
                            "artists": [
                                {
                                    "name": "My artist"
                                }
                            ]
                        }
                    }
                ]
            }
        "#;
        let test_tracks_page: TracksPage = serde_json::from_str(&test_tracks_page_str).unwrap();
        assert_eq!(test_tracks_page.items.len(), 1);
        assert_eq!(test_tracks_page.items[0].track.name, "My track");
    }

    #[test]
    fn normalize_track_name() {
        assert_eq!(
            normalize_keyword(r#"There's  "A T*i*t*l*e" —–- Gün-ther &  $imon Remix"#),
            normalize_keyword(r#"theres "a title" gunther and simon remix"#)
        );
    }
}
