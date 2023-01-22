use reqwest::StatusCode;
use serenity::client::Context;

use crate::data::ReqwestClientContainer;

use serde::Deserialize;

lazy_static::lazy_static! {
    static ref LYRIC_API_KEY: String = std::env::var("LYRIC_API_KEY").expect("Expected LYRIC_API_KEY in dotenv file");
}

#[cfg(test)]
mod tests {

    use super::{LyricResponse, LYRIC_API_KEY};
    #[tokio::test]
    async fn output_lyric_requst() {
        dotenv::dotenv().ok();

        let client = reqwest::Client::new();

        let res = client
            .get("https://api.ksoft.si/lyrics/search")
            .bearer_auth(LYRIC_API_KEY.clone())
            .query(&[("q", "fuck you ceelo green"), ("limit", "1")])
            .header("User-Agent", "DJBOT_TEST_OUTPUT_LYRIC_REQUEST")
            .send()
            .await;

        assert!(res.is_ok());

        let res = res.unwrap();

        let code = res.status();

        assert!(code.is_success());

        println!("{:?}", res.text().await);
    }

    #[tokio::test]
    async fn map_lyric_to_lyric_response() {
        dotenv::dotenv().ok();

        let client = reqwest::Client::new();

        let res = client
            .get("https://api.ksoft.si/lyrics/search")
            .bearer_auth(LYRIC_API_KEY.clone())
            .query(&[("q", "fuck you ceelo green"), ("limit", "1")])
            .header("User-Agent", "DJBOT_TEST_MAP_LYRIC_TO_LYRIC_RESPONSE")
            .send()
            .await;

        assert!(res.is_ok());

        let res = res.unwrap();

        let code = res.status();

        assert!(code.is_success());

        let data: LyricResponse = res.json().await.unwrap();

        println!("{data:?}");
    }
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
struct LyricResponse {
    data: Vec<Data>,
}

#[derive(Debug, Deserialize, Clone)]
#[non_exhaustive]
pub struct Data {
    pub lyrics: String,
    pub name: String,
    pub artist: String,
}

pub async fn get_lyrics(ctx: &Context, search: String) -> anyhow::Result<Option<Data>> {
    let data = ctx.data.read().await;
    let client = data.get::<ReqwestClientContainer>().unwrap().clone();

    let res = client
        .get("https://api.ksoft.si/lyrics/search")
        .bearer_auth(LYRIC_API_KEY.clone())
        .query(&[("q", search.as_str()), ("limit", "1")])
        .header("User-Agent", "dj-bot")
        .send()
        .await?;

    if let StatusCode::NOT_FOUND = res.status() {
        Ok(None)
    } else {
        let res = res.error_for_status()?;

        let data: LyricResponse = res.json().await?;

        let song_data = data.data[0].clone();

        Ok(Some(song_data))
    }
}
