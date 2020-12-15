use std::fmt::Formatter;

use serde::Deserialize;

use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct PlayListResponse {
    pub url: String,
}

#[derive(Debug)]
pub enum PlayListError {
    ListOfUrlsError(Vec<u8>),
}

impl std::fmt::Display for PlayListError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ListOfUrlsError(e) => {
                let string_err = String::from_utf8(e.clone());
                write!(f, "ListOfUrlsError: {:?}", string_err)
            }
        }
    }
}

impl std::error::Error for PlayListError {}

pub async fn get_list_of_urls(url: String) -> anyhow::Result<Vec<PlayListResponse>> {
    let output = Command::new("youtube-dl")
        .args(&["-j", "--flat-playlist", &url])
        .output()
        .await?;

    if !output.status.success() {
        Err(PlayListError::ListOfUrlsError(output.stderr).into())
    } else {
        let output = String::from_utf8(output.stdout)?;
        let mut json_output = vec![];
        for line in output.lines() {
            let json: PlayListResponse = serde_json::from_str(line)?;
            json_output.push(json);
        }

        Ok(json_output)
    }
}
