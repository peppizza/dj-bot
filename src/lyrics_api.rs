lazy_static::lazy_static! {
    static ref LYRIC_API_KEY: String = std::env::var("LYRIC_API_KEY").expect("Expected LYRIC_API_KEY in dotenv file");
}

#[cfg(test)]
mod tests {

    use super::LYRIC_API_KEY;
    #[tokio::test]
    async fn output_lyric_requst() {
        dotenv::dotenv().ok();

        let client = reqwest::Client::new();

        let res = client
            .get("https://api.ksoft.si/lyrics/search")
            .bearer_auth(LYRIC_API_KEY.clone())
            .query(&[("q", "the real slim shady"), ("limit", "1")])
            .send()
            .await;

        assert!(res.is_ok());

        let res = res.unwrap();

        let code = res.status();

        let body = res.text().await.unwrap();

        assert!(code.is_success());
    }
}
