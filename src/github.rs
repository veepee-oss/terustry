use std::str;

use anyhow::Result;
use handlebars::Handlebars;
use reqwest::Client;
use serde::Deserialize;

use crate::conf;
use crate::conf::ConfigurationProvider;

#[derive(Debug, Deserialize)]
pub struct GithubReleaseListResponse {
    #[serde()]
    pub name: String,
    #[serde()]
    pub tag_name: String,
}

pub async fn versions(client: &Client, provider: &ConfigurationProvider) -> Result<Vec<String>> {
    Ok(client
        .get(provider.version.uri.to_string())
        .header(
            reqwest::header::AUTHORIZATION,
            format!(
                "Token {}",
                Handlebars::new()
                    .render_template(&provider.version.token, &conf::os_env_hashmap())?
            ),
        )
        .header("User-Agent", "terustry")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<GithubReleaseListResponse>>()
        .await?
        .iter()
        .map(|element| element.tag_name.to_owned())
        .collect())
}
