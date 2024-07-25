use std::time::Duration;

use anyhow::Result;
use moka::future::Cache;
use reqwest::Client;

use crate::conf::ConfigurationProvider;
use crate::{github, gitlab};

pub struct VersionClient {
    versions_cache: Cache<String, Vec<String>>,
}

impl VersionClient {
    pub fn new() -> Self {
        Self {
            versions_cache: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(5))
                .build()
        }
    }

    pub async fn invalidate(&self, provider: &ConfigurationProvider) {
        let key = format!("get_version_{}", provider.name);
        self.versions_cache.invalidate(&key).await
    }

    pub async fn get_versions(
        &self,
        client: &Client,
        provider: &ConfigurationProvider,
    ) -> Result<Vec<String>> {
        let key = format!("get_version_{}", provider.name);
        let result = self
            .versions_cache
            .try_get_with(key, fetch_versions(&client, &provider));
        match result.await {
            Ok(vec) => Ok(vec),
            Err(_) => anyhow::bail!(format!("could not get versions for {}", provider.name))
        }
    }
}

pub async fn fetch_versions(
    client: &Client,
    provider: &ConfigurationProvider,
) -> Result<Vec<String>> {
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
        s => anyhow::bail!(format!("provider type {} not found", s)),
    }
}
