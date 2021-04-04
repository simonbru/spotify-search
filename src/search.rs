use std::fs;
use std::path::Path;

use serde::{Deserialize, Deserializer};
use unidecode::unidecode;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Image {
    pub height: i32,
    pub width: i32,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Album {
    pub name: String,
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Track {
    pub uri: String,
    pub name: String,
    pub album: Album,

    #[serde(deserialize_with = "exclude_invalid_artists")]
    pub artists: Vec<Artist>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct TrackMeta {
    pub added_at: String,
    pub track: Track,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct TracksPage {
    #[serde(deserialize_with = "exclude_null_tracks")]
    pub items: Vec<TrackMeta>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub tracks: TracksPage,
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

fn match_track(track: &Track, keywords: &[&str]) -> bool {
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

pub struct SearchResult {
    pub collection: String,
    pub index: usize,
    pub track: TrackMeta,
}

fn search_in_tracks(
    collection: &str,
    tracks: Vec<TrackMeta>,
    keywords: &[&str],
) -> Vec<SearchResult> {
    tracks
        .into_iter()
        .enumerate()
        .filter(|(_, track_meta)| match_track(&track_meta.track, keywords))
        .map(|(i, track_meta)| SearchResult {
            collection: collection.to_string(),
            index: i,
            track: track_meta,
        })
        .collect()
}

pub fn search(library_path: &Path, search_keywords: &[&str]) -> Vec<SearchResult> {
    // TODO: Hoist all those panics
    let mut results: Vec<SearchResult> = vec![];

    // Search tracks in library
    let mut search_in_library = || {
        let path = Path::new(library_path).join("tracks.json");
        let contents = fs::read_to_string(&path).expect(&format!("Could not read {:?}", path));
        let library_tracks: LibraryTracks = match serde_json::from_str(&contents) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Could not parse {:?}: {}", path.file_name().unwrap(), err);
                return;
            }
        };
        let tracks = search_in_tracks("Library", library_tracks, &search_keywords);
        results.extend(tracks);
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
        let tracks = search_in_tracks(&playlist.name, playlist.tracks.items, &search_keywords);
        results.extend(tracks);
    }
    return results;
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
                                ],
                                "album": {
                                    "name": "My album",
                                    "images": []
                                }
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
                        album: Album {
                            name: "My album".to_string(),
                            images: vec![],
                        }
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
                ],
                "album": {
                    "name": "Album",
                    "images": []
                }
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
                            ],
                            "album": {
                                "name": "Album",
                                "images": []
                            }
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
