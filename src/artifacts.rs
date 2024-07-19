use anyhow::Result;
use handlebars::Handlebars;
use reqwest::Client;
use std::collections::HashMap;

use crate::conf::ConfigurationProvider;
use crate::models::{DownloadResponse, GpgPublicKey, SigningKey};

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

#[cached::proc_macro::cached(
    result = true,
    result_fallback = true,
    time = 600,
    key = "String",
    convert = r#"{ format!("get_artifacts_{}_{}_{}_{}", provider.name, version, os, arch) }"#
)]
pub async fn get(
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
            log::warn!("could not fetch artefact {}/{}/{}/{}: {}",
                provider.name, version, os, arch, err);
            Err(err)
        }
    }
}
