pub mod config;

use config::ConfigManager;
use derive_builder::Builder;
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use std::{borrow::Cow, collections::HashMap};

/// The serialized identifier for passing to `retrieve()`.
pub type SerializedIdentifier = String;

/// The search mode.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SearchMode {
    /// Return the first response.
    /// 
    /// For example, `["a", "b", "c"]` and `"c"` returns the fast,
    /// we return `"c"`.
    /// 
    /// This is the default mode.
    FastFirst,
    /// Return according to the order of the response.
    /// 
    /// For example, even if `["a", "b", "c"]` and `"c"` returns the fast,
    /// we still wait for `"a"` and return `"a"`. If `"a"` has no result,
    /// we return `"b"`.
    OrderFirst,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::FastFirst
    }
}

/// The metadata of the artist of a song.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[non_exhaustive]
pub struct Artist {
    /// The identifier of this artist.
    #[builder(default = "".to_string())]
    pub id: String,
    /// The name of this artist.
    pub name: String,
}

/// The metadata of the album of a song.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[non_exhaustive]
pub struct Album {
    /// The identifier of this artist.
    #[builder(default = "".to_string())]
    pub id: String,
    /// The name of this album.
    pub name: String,
}

/// The metadata of a song.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[non_exhaustive]
pub struct Song {
    /// The identifier of this song.
    #[builder(default = "".to_string())]
    pub id: String,
    /// The name of this song.
    pub name: String,
    /// The duration of this song (ms).
    #[builder(default)]
    pub duration: Option<i64>,
    /// The artist of this song.
    #[builder(default)]
    pub artists: Vec<Artist>,
    /// The album of this song.
    #[builder(default)]
    pub album: Option<Album>,
    /// The context of this song.
    ///
    /// For example, the URI identifier of this song.
    #[builder(default)]
    pub context: Option<HashMap<String, String>>,
}

/// The song identifier with the engine information.
#[derive(Clone, Serialize, Deserialize, TypedBuilder)]
#[non_exhaustive]
pub struct SongSearchInformation {
    /// The retrieve source of this song, for example: `bilibili`.
    pub source: Cow<'static, str>,
    /// The serialized identifier of this song.
    pub identifier: SerializedIdentifier,
    /// The details of this song.
    #[builder(default)]
    pub song: Option<Song>,
}

/// The information of the song retrieved with `retrieve()`.
#[derive(Clone, Serialize, Deserialize, TypedBuilder)]
#[non_exhaustive]
pub struct RetrievedSongInfo {
    /// The retrieve source of this song, for example: `bilibili`.
    pub source: Cow<'static, str>,
    /// The URL of this song.
    pub url: String,
}

/// The context.
#[derive(Clone, Default, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option), default)]
#[non_exhaustive]
pub struct Context {
    /// The proxy URI
    ///
    /// For example: `https://secure.example` or
    /// `"socks5://192.168.1.1:9000"`
    pub proxy_uri: Option<String>,

    /// Whether to enable FLAC support.
    pub enable_flac: bool,

    /// The search mode for waiting the response.
    pub search_mode: SearchMode,

    /// The config for engines.
    pub config: Option<ConfigManager>,
}

impl Context {
    pub fn try_get_proxy(&self) -> reqwest::Result<Option<Proxy>> {
        self.proxy_uri.clone().map(Proxy::all).transpose()
    }
}

impl std::fmt::Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.keyword())
    }
}

impl Song {
    /// Generate the keyword of this song.
    pub fn keyword(&self) -> String {
        // {Song Name}
        let mut keyword = self.name.to_string();

        if self.artists.is_empty() {
            return keyword;
        }

        let max_idx = self.artists.len() - 1;

        // Add hyphen between the song name and the following artist name.
        keyword.push_str(" - ");

        for (idx, artist) in self.artists.iter().enumerate() {
            // "[keyword] {artist.name}"
            keyword.push_str(&artist.name);

            if idx != max_idx {
                // ", " if this is not the last item.
                keyword.push_str(", ");
            }
        }

        // {Song name} - {Artist 1's name}, {Artist 2's name}[, ...]
        keyword
    }
}

#[cfg(test)]
mod tests {
    use crate::{Artist, Song};

    #[test]
    fn test_keyword_with_no_artist() {
        let s = Song {
            id: "114514".to_string(),
            name: "Lost River".to_string(),
            artists: vec![],
            ..Default::default()
        };

        assert_eq!(s.keyword(), "Lost River");
    }

    #[test]
    fn test_keyword_with_single_artist() {
        let s = Song {
            id: "123".to_string(),
            name: "TT".to_string(),
            artists: vec![Artist {
                id: "114".to_string(),
                name: "Twice".to_string(),
            }],
            ..Default::default()
        };

        assert_eq!(s.keyword(), "TT - Twice");
    }

    #[test]
    fn test_keyword_with_multiple_artist() {
        let s = Song {
            id: "123".to_string(),
            name: "Hope for Tomorrow - Melchi Remix".to_string(),
            artists: vec![
                Artist {
                    id: "1".to_string(),
                    name: "Alex H".to_string(),
                },
                Artist {
                    id: "2".to_string(),
                    name: "Z8phyR".to_string(),
                },
                Artist {
                    id: "3".to_string(),
                    name: "Melchi".to_string(),
                },
            ],
            ..Default::default()
        };

        assert_eq!(
            s.keyword(),
            "Hope for Tomorrow - Melchi Remix - Alex H, Z8phyR, Melchi"
        );
    }
}
