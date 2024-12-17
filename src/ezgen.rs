use bytes::Bytes;

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Request data to the ezgen API
#[derive(Serialize)]
struct RequestData {
    guidance_scale: u32,
    height: u32,
    negative_prompt: String,
    num_inference_steps: u32,
    prompt: String,
    width: u32,
}

/// An error detail from the ezgen API
#[derive(Deserialize, Debug, thiserror::Error)]
#[error("EzgenError")]
pub struct EzgenError {
    pub error: String,
    pub message: Option<String>,
}

/// Either a HTTP error or an API error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ezgen(#[from] EzgenError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

/// Generate an image by sending `prompt` to the API, authenticating with `key`.
pub async fn get_image(key: &str, prompt: &str) -> Result<Bytes, Error> {
    let request_data = RequestData {
        guidance_scale: 0,
        height: 768,
        negative_prompt: String::new(),
        num_inference_steps: 4,
        prompt: prompt.to_string(),
        width: 1360,
    };

    let client = Client::new();
    let response = client
        .post("https://ezgen.net/api/v0/flux/generate")
        .header("Authorization", key)
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()
        .await?
        .error_for_status()?;

    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok());

    // Check if we got an image or an error
    if let Some("image/webp") = content_type {
        Ok(response.bytes().await?)
    } else {
        let err: EzgenError = response.json().await?;
        Err(Error::Ezgen(err))
    }
}
