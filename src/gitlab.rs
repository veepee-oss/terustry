use anyhow::Result;
use handlebars::Handlebars;
use reqwest::Client;
use serde::Deserialize;
use std::str;

use crate::conf;
use crate::conf::ConfigurationProvider;

#[derive(Debug, Deserialize)]
pub struct GitlabReleaseListResponse {
    #[serde()]
    pub name: String,
    #[serde()]
    pub tag_name: String,
}

pub async fn versions(client: &Client, provider: &ConfigurationProvider) -> Result<Vec<String>> {
    Ok(client
        .get(format!("{}?per_page=100", provider.version.uri))
        .header(
            "PRIVATE-TOKEN",
            Handlebars::new().render_template(&provider.version.token, &conf::os_env_hashmap())?
        )
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<GitlabReleaseListResponse>>()
        .await?
        .iter()
        .map(|element| element.tag_name.to_owned())
        .collect())
}
