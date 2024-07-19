use reqwest::Client;

use crate::conf::ConfigurationProvider;
use crate::{github, gitlab};

#[cached::proc_macro::cached(
    result = true,
    result_fallback = true,
    time = 600,
    key = "String",
    convert = r#"{ format!("get_versions_{}", provider.name) }"#
)]
pub async fn get_versions(
    client: &Client,
    provider: &ConfigurationProvider,
) -> anyhow::Result<Vec<String>> {
    match provider.version.kind.as_str() {
        "gitlab" => match gitlab::versions(client, provider).await {
            Ok(vec) => Ok(vec),
            Err(err) => {
                log::warn!("could not get gitlab versions: {}", err);
                Err(err)
            }
        },
        "github" => match github::versions(client, provider).await {
            Ok(vec) => Ok(vec),
            Err(err) => {
                log::warn!("could not get github versions: {}", err);
                Err(err)
            }
        },
        s => anyhow::bail!(format!("Provider type {} not found", s)),
    }
}
