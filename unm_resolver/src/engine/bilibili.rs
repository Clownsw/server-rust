//! UNM Resolver [Engine]: Bilibili
//!
//! It can fetch audio from Bilibili Music.

use async_trait::async_trait;
use rayon::prelude::*;

use std::str::FromStr;

use crate::engine::Json;
use crate::request::request;
use crate::utils::UnableToExtractJson;
use http::Method;
use url::Url;
use urlencoding::encode;

use super::{similar_song_selector_constructor, Artist, Context, Engine, Song};

/// The `bilibili` engine that can fetch audio from Bilibili Music.
pub struct BilibiliEngine;

#[async_trait]
impl Engine for BilibiliEngine {
    async fn check<'a>(&self, info: &'a Song, ctx: &'a Context) -> anyhow::Result<Option<String>> {
        match search(info, ctx).await? {
            None => Ok(None),
            Some(id) => Ok(track(id, ctx).await?),
        }
    }
}

/// Get search data from Bilibili Music.
async fn get_search_data(keyword: &str, ctx: &Context<'_>) -> anyhow::Result<Json> {
    let url_str = format!(
        "https://api.bilibili.com/audio/music-service-c/s?\
        search_type=music&page=1&pagesize=30&\
        keyword=${0}",
        encode(keyword)
    );
    let url = Url::from_str(&url_str)?;

    let res = request(Method::GET, &url, None, None, ctx.proxy.cloned()).await?;
    Ok(res.json().await?)
}

/// Track the ID from Bilibili Music.
async fn get_tracked_data(id: &str, ctx: &Context<'_>) -> anyhow::Result<Json> {
    let url_str = format!(
        "https://www.bilibili.com/audio/music-service-c/web/url?rivilege=2&quality=2&sid={id}"
    );
    let url = Url::from_str(&url_str)?;

    let res = request(Method::GET, &url, None, None, ctx.proxy.cloned()).await?;
    Ok(res.json().await?)
}

/// Find the matched song.
///
/// `data` is the `data/result` of the Bilibili Music response.
async fn find_match(info: &Song, data: &[Json]) -> anyhow::Result<Option<String>> {
    let selector = similar_song_selector_constructor(info).1;
    let similar_song = data
        .par_iter()
        .map(|entry| format(entry).ok())
        .find_first(|s| selector(&s))
        .expect("should be Some");

    Ok(similar_song.map(|song| song.id))
}

/// Search and get the audio ID from Bilibili Music.
async fn search(info: &Song, ctx: &Context<'_>) -> anyhow::Result<Option<String>> {
    let response = get_search_data(&info.keyword(), ctx).await?;
    let result = response
        .pointer("/data/result")
        .ok_or(anyhow::anyhow!("/data/result not found"))?
        .as_array()
        .ok_or(UnableToExtractJson("/data/result", "array"))?;

    let matched = find_match(info, result).await?;
    Ok(matched)
}

/// Track the song with the audio ID.
async fn track(id: String, ctx: &Context<'_>) -> anyhow::Result<Option<String>> {
    let response = get_tracked_data(&id, ctx).await?;
    let links = response
        .pointer("/data/cdns")
        .ok_or(anyhow::anyhow!("/data/cdns not found"))?
        .as_array()
        .ok_or(UnableToExtractJson("/data/cdns", "array"))?;

    if links.is_empty() {
        return Ok(None);
    }

    let link = links[0]
        .as_str()
        .ok_or(UnableToExtractJson("/data/cdns/0", "string"))?
        .replace("https", "http");

    Ok(Some(link))
}

/// Format the Bilibili song metadata to [`Song`].
fn format(song: &Json) -> anyhow::Result<Song> {
    let id = song["id"]
        .as_i64()
        .ok_or(UnableToExtractJson("/id", "i64"))?;
    let name = song["title"]
        .as_str()
        .ok_or(UnableToExtractJson("/title", "string"))?;
    let mid = song["mid"]
        .as_i64()
        .ok_or(UnableToExtractJson("/mid", "i64"))?;
    let author = song["author"]
        .as_str()
        .ok_or(UnableToExtractJson("/author", "string"))?;
    let x = Song {
        id: id.to_string(),
        name: String::from(name),
        artists: vec![Artist {
            id: mid.to_string(),
            name: String::from(author),
        }],
        ..Default::default()
    };
    Ok(x)
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use super::*;

    fn get_info_1() -> Song {
        // https://music.163.com/api/song/detail?ids=[385552]
        Song {
            name: String::from("干杯"),
            artists: vec![Artist {
                name: String::from("五月天"),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    async fn bilibili_search() {
        let info = get_info_1();
        let id = search(&info, &Context::default()).await.unwrap();
        assert_eq!(id, Some("349595".to_string()));
    }

    #[test]
    async fn bilibili_track() {
        let url = track("349595".to_string(), &Context::default())
            .await
            .unwrap()
            .unwrap();
        println!("{}", url);
    }

    #[test]
    async fn bilibili_check() {
        let p = BilibiliEngine;
        let info = get_info_1();
        let url = p.check(&info, &Context::default()).await.unwrap().unwrap();
        println!("{}", url);
    }
}
