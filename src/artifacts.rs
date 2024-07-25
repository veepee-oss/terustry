use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use handlebars::Handlebars;
use moka::future::Cache;
use reqwest::Client;

use crate::conf::ConfigurationProvider;
use crate::models::{DownloadResponse, GpgPublicKey, SigningKey};

pub struct ArtifactClient {
    artifact_cache: Cache<String, DownloadResponse>,
}

impl ArtifactClient {
    pub fn new() -> Self {
        Self {
            artifact_cache: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(5))
                .build(),
        }
    }

    pub async fn invalidate(
        &self,
        provider: ConfigurationProvider,
        version: String,
        os: String,
        arch: String,
    ) {
        let key = format!(
            "get_artifacts_{}_{}_{}_{}",
            provider.name, version, os, arch
        );
        self.artifact_cache.invalidate(&key).await
    }

    pub async fn get(
        &self,
        client: &Client,
        provider: ConfigurationProvider,
        version: String,
        os: String,
        arch: String,
    ) -> Result<DownloadResponse> {
        let key = format!(
            "get_artifacts_{}_{}_{}_{}",
            provider.name, version, os, arch
        );
        let result = self
            .artifact_cache
            .try_get_with(key, fetch(client, provider, version, os, arch));
        match result.await {
            Ok(vec) => Ok(vec),
            // NOTE "could not fetch artefact {}/{}/{}/{}", provider.name, version, os, arch
            Err(_) => anyhow::bail!(format!("could not fetch artefact")),
        }
    }
}

pub async fn sha(
    client: &Client,
    uri: String,
    version: &String,
    os: &String,
    arch: &String,
) -> Result<String> {
    Ok(String::from(
        client
            .get(uri)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .split('\n')
            .find(|&x| x.contains(format!("{}_{}", os, arch).as_str()))
            .map_or(
                Err(anyhow::anyhow!(
                    "Line not found for {} {} {}",
                    version,
                    os,
                    arch
                )),
                |line| {
                    line.split(' ')
                        .find(|e| e.len() == 64)
                        .ok_or(anyhow::anyhow!(format!(
                            "Can't parse line [{}] to extract sha",
                            line
                        )))
                },
            )?,
    ))
}

pub async fn fetch(
    client: &Client,
    provider: ConfigurationProvider,
    version: String,
    os: String,
    arch: String,
) -> Result<DownloadResponse> {
    let hb = Handlebars::new();
    let conf: HashMap<String, String> = HashMap::from([
        (String::from("version"), version.to_owned()),
        (String::from("os"), os.to_owned()),
        (String::from("arch"), arch.to_owned()),
    ]);

    let shasum = sha(
        client,
        hb.render_template(&provider.artifact.shasums_url, &conf)?,
        &version,
        &os,
        &arch,
    ).await;

    match shasum {
        Ok(fetched_sum) => Ok(DownloadResponse {
            protocols: provider.protocols,
            os: os.to_owned(),
            arch: arch.to_owned(),
            filename: hb.render_template(&provider.artifact.filename, &conf)?,
            download_url: hb.render_template(&provider.artifact.download_url, &conf)?,
            shasums_url: hb.render_template(&provider.artifact.shasums_url, &conf)?,
            shasums_signature_url: hb
                .render_template(&provider.artifact.shasums_signature_url, &conf)?,
            shasum: fetched_sum,
            signing_keys: SigningKey {
                gpg_public_keys: vec![GpgPublicKey {
                    key_id: provider.signature.key_id,
                    ascii_armor: provider.signature.key_armor,
                }],
            },
        }),
        Err(err) => {
            log::warn!("could not fetch artifact {}/{}/{}/{}: {}",
                provider.name, version, os, arch, err);
            Err(err)
        }
    }
}
