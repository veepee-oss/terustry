use std::collections::HashMap;
use std::error::Error;
use std::str;

use actix_web::client::Client;
use actix_web::http::{header, StatusCode};
use handlebars::Handlebars;
use async_recursion::async_recursion;

use crate::conf::ConfigurationProvider;
use crate::models::{DownloadResponse, GpgPublicKey, SigningKey};

#[async_recursion(?Send)]
pub async fn sha(uri: String, version: String, os: String, arch: String) -> Result<String, Box<dyn Error>> {
    let mut response = Client::new().get(uri).send().await?;
    match response.status() {
        StatusCode::FOUND => sha(response.headers().get(header::LOCATION).unwrap().to_str().unwrap().to_string(), version, os, arch).await,
        StatusCode::OK => Ok(
            String::from(String::from(str::from_utf8(&response.body().await?)?)
                .split('\n')
                .into_iter()
                .find(|&x| x.contains(format!("{}_{}", os, arch).as_str()))
                .map_or(Err(format!("Line not found for {} {} {}", version, os, arch)), |line| {
                    line.split(' ')
                        .into_iter()
                        .filter(|e| e.len() == 64)
                        .next()
                        .ok_or(String::from(format!("Can't parse line [{}] to extract sha", line)))
                })?
            )
        ),
        s => Err(String::from(format!("Artifacts response {}", s)).into()),
    }
}

pub async fn get(provider: ConfigurationProvider, version: String, os: String, arch: String) -> Result<DownloadResponse, Box<dyn Error>> {
    let hb = Handlebars::new();
    let conf: HashMap<String, String> = HashMap::from([
        (String::from("version"), version.to_owned()),
        (String::from("os"), os.to_owned()),
        (String::from("arch"), arch.to_owned()),
    ]);
    Ok(DownloadResponse {
        protocols: provider.protocols,
        os: os.to_owned(),
        arch: arch.to_owned(),
        filename: hb.render_template(&provider.artifact.filename, &conf)?,
        download_url: hb.render_template(&provider.artifact.download_url, &conf)?,
        shasums_url: hb.render_template(&provider.artifact.shasums_url, &conf)?,
        shasums_signature_url: hb.render_template(&provider.artifact.shasums_signature_url, &conf)?,
        shasum: sha(hb.render_template(&provider.artifact.shasums_url, &conf)?, version, os, arch).await?,
        signing_keys: SigningKey {
            gpg_public_keys: vec![
                GpgPublicKey {
                    key_id: provider.signature.key_id,
                    ascii_armor: provider.signature.key_armor,
                }
            ]
        },
    })
}
