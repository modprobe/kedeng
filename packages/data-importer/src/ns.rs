use anyhow::Result;
use reqwest::ClientBuilder;
use reqwest::header::{HeaderMap, HeaderValue};

pub(crate) fn create_ns_api_client(api_key: &str) -> Result<reqwest::Client> {
    let mut api_key_header = HeaderValue::from_str(api_key)?;
    api_key_header.set_sensitive(true);

    let mut headers = HeaderMap::new();
    headers.insert("Ocp-Apim-Subscription-Key", api_key_header);

    Ok(ClientBuilder::new().default_headers(headers).user_agent("kedeng/0.1").build()?)
}
