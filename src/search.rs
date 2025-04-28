use std::fs;
use std::path::Path;

use unidecode::unidecode;

mod raw {
    use serde::Deserialize;

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

        pub artists: Vec<Artist>,
    }

    #[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
    pub struct TrackMeta {
        pub added_at: String,
        /// Some playlist contain items with dummy metadata and "track: null"
        pub track: Option<Track>,
    }

    #[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
    pub struct TracksPage {
        pub items: Vec<TrackMeta>,
    }

    #[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
    pub struct Playlist {
        pub uri: String,
        pub name: String,
        pub tracks: TracksPage,
    }
}

// TODO: different type for "artist with empty name" ? or different type for "track with invalid artists" ?
pub use raw::Track;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TrackMeta {
    pub added_at: String,
    pub position: u32,
    pub track: Track,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TracksPage {
    pub items: Vec<TrackMeta>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Playlist {
    pub uri: String,
    pub name: String,
    pub tracks: TracksPage,
}

impl Track {
    fn from_raw(track: raw::Track) -> Self {
        Track {
            uri: track.uri,
            name: track.name,
            album: track.album,
            // When a track has no artist, its list of artist contains a single artist with empty values.
            artists: track
                .artists
                .into_iter()
                .filter(|artist| artist.name != "")
                .collect(),
        }
    }
}

fn from_raw_track_metas(tracks: Vec<raw::TrackMeta>) -> Vec<TrackMeta> {
    tracks
        .into_iter()
        .filter(|track_meta| track_meta.track.is_some())
        .enumerate()
        .map(|(i, track_meta)| {
            let position = u32::try_from(i).unwrap() + 1;
            TrackMeta {
                added_at: track_meta.added_at,
                position,
                track: Track::from_raw(track_meta.track.unwrap()),
            }
        })
        .collect()
}

impl From<raw::Playlist> for Playlist {
    fn from(playlist: raw::Playlist) -> Self {
        Playlist {
            uri: playlist.uri,
            name: playlist.name,
            tracks: TracksPage {
                items: from_raw_track_metas(playlist.tracks.items),
            },
        }
    }
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

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub collection: Collection,
    pub track: TrackMeta,
}

fn search_in_tracks(
    collection: &Collection,
    tracks: &Vec<TrackMeta>,
    keywords: &[&str],
) -> Vec<SearchResult> {
    tracks
        .iter()
        .filter(|track_meta| match_track(&track_meta.track, keywords))
        .map(|track_meta| SearchResult {
            collection: collection.clone(),
            track: track_meta.clone(),
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
        let library_tracks = match serde_json::from_str::<Vec<raw::TrackMeta>>(&contents) {
            Ok(val) => from_raw_track_metas(val),
            Err(err) => {
                eprintln!("Could not parse {:?}: {}", path.file_name().unwrap(), err);
                return;
            }
        };
        let collection = Collection {
            name: "Library".to_string(),
            uri: "spotify:collection:tracks".to_string(),
        };
        let tracks = search_in_tracks(&collection, &library_tracks, &search_keywords);
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

        let playlist: Playlist = match serde_json::from_str::<raw::Playlist>(&contents) {
            Ok(val) => val.into(),
            Err(err) => {
                eprintln!("Could not parse {:?}: {}", path.file_name().unwrap(), err);
                continue;
            }
        };
        let collection = Collection {
            name: playlist.name,
            uri: playlist.uri,
        };
        let tracks = search_in_tracks(&collection, &playlist.tracks.items, &search_keywords);
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
                "uri": "spotify:playlist:37i9dQZF1DX0aSJooo0zWR",
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
        let test_playlist: Playlist = serde_json::from_str::<raw::Playlist>(&test_playlist_str)
            .unwrap()
            .into();
        let expected_playlist = Playlist {
            name: "my_playlist".to_string(),
            uri: "spotify:playlist:37i9dQZF1DX0aSJooo0zWR".to_string(),
            tracks: TracksPage {
                items: vec![TrackMeta {
                    added_at: "2010-08-23T10:33:01Z".to_string(),
                    position: 1,
                    track: Track {
                        uri: "spotify:track:asdfasdf".to_string(),
                        name: "My track".to_string(),
                        artists: vec![Artist {
                            name: "My artist".to_string(),
                        }],
                        album: Album {
                            name: "My album".to_string(),
                            images: vec![],
                        },
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
        let test_track = Track::from_raw(serde_json::from_str(&test_track_str).unwrap());
        assert_eq!(test_track.artists.len(), 0)
    }

    #[test]
    fn parse_track_metas_exclude_null_tracks() {
        let test_tracks_page_str = r#"
            [
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
        "#;
        let test_track_metas =
            from_raw_track_metas(serde_json::from_str(&test_tracks_page_str).unwrap());
        assert_eq!(test_track_metas.len(), 1);
        assert_eq!(test_track_metas[0].track.name, "My track");
    }

    #[test]
    fn normalize_track_name() {
        assert_eq!(
            normalize_keyword(r#"There's  "A T*i*t*l*e" —–- Gün-ther &  $imon Remix"#),
            normalize_keyword(r#"theres "a title" gunther and simon remix"#)
        );
    }
}
