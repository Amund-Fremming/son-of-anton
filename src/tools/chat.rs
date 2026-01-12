use std::env;

use crate::app_error::AppError;

#[derive(Debug)]
pub struct ChatClient {
    client: reqwest::Client,
    base_url: String,
}

impl ChatClient {
    pub fn new(client: reqwest::Client) -> Result<Self, AppError> {
        let base_url = env::var("CHAT_CLIENT_BASE_URL")?;
        Ok(Self { client, base_url })
    }
}
