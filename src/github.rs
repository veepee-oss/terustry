use std::error::Error;
use std::str;

use actix_web::client::Client;
use actix_web::http;
use actix_web::http::StatusCode;
use handlebars::Handlebars;
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

pub async fn versions(provider: &ConfigurationProvider) -> Result<Vec<String>, Box<dyn Error>> {
    let token = Handlebars::new().render_template(&provider.version.token, &conf::os_env_hashmap())?;

    let mut response = Client::default()
        .get(format!("{}", provider.version.uri))
        .header(http::header::AUTHORIZATION, format!("Token {}", token,))
        .header("User-Agent","terustry")
        .send().await?;
    match response.status() {
        StatusCode::OK => Ok(serde_json::from_str::<Vec<GithubReleaseListResponse>>(str::from_utf8(&response.body().await?)?)?.iter().map(|element| element.tag_name.to_owned()).collect()),
        s => Err(String::from(format!("Github response {}", s)).into()),
    }
}
