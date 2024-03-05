use reqwest::Client;

use crate::conf::ConfigurationProvider;
use crate::{github, gitlab};

#[cached::proc_macro::cached(
    result = true,
    sync_writes = true,
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
        "gitlab" => gitlab::versions(client, provider).await,
        "github" => github::versions(client, provider).await,
        s => anyhow::bail!(format!("Provider type {} not found", s)),
    }
}
