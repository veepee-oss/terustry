use std::sync::Arc;

use crate::conf::{Configuration, ConfigurationProvider};
use crate::errors::AppError;
use crate::models::*;
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::routing::get;

use axum::{Json, Router};
use clap::Parser;
use reqwest::Client;
use tracing::info;

mod artifacts;
mod conf;
mod errors;
mod github;
mod gitlab;
mod models;
mod versions;

async fn root() -> Json<Root> {
    Json(Root { ok: true })
}

async fn well_known() -> Json<WellKnown> {
    Json(WellKnown {
        providers_v1: "/terraform/providers/v1/".to_string(),
    })
}

fn get_provider_conf(
    conf: &Configuration,
    namespace: String,
    name: String,
) -> anyhow::Result<ConfigurationProvider> {
    conf.providers
        .iter()
        .find(|&p| p.name == format!("{}/{}", namespace, name))
        .cloned()
        .ok_or(anyhow!(format!("Provider {} not found", name)))
}

async fn versions(
    State(data): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<VersionsResponse>, AppError> {
    info!("list versions of {}/{}", namespace, name);
    let provider = get_provider_conf(&data.conf, namespace, name)?;
    Ok(Json(VersionsResponse {
        id: provider.name.clone(),
        versions: versions::get_versions(&data.client, &provider)
            .await
            .inspect_err(|e| log::error!("Error getting versions: {}", e))?
            .iter()
            .map(|version| Version {
                version: version.to_owned().trim_start_matches('v').to_string(),
                protocols: provider.protocols.to_owned(),
                platforms: provider
                    .binaries
                    .iter()
                    .cloned()
                    .map(|binary| VersionPlatform {
                        os: binary.os,
                        arch: binary.arch,
                    })
                    .collect::<Vec<VersionPlatform>>(),
            })
            .collect(),
    }))
}

async fn download(
    State(data): State<Arc<AppState>>,
    Path((namespace, name, version, os, arch)): Path<(String, String, String, String, String)>,
) -> Result<Json<DownloadResponse>, AppError> {
    info!("download {}/{}/{} for {}/{}", namespace, name, version, os, arch);
    Ok(Json(
        artifacts::get(
            &data.client,
            get_provider_conf(&data.conf, namespace, name)?,
            version,
            os,
            arch,
        )
        .await
        .inspect_err(|e| log::error!("Error download: {}", e))?,
    ))
}

struct AppState {
    conf: Configuration,
    client: Client,
}

#[derive(Parser, Debug)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(short, long, default_value = "/etc/terustry.yml")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    tracing_subscriber::fmt().json().init();
    log::info!("Starting terustry");

    let app = Router::new()
        .route("/", get(root))
        .route("/.well-known/terraform.json", get(well_known))
        .route(
            "/terraform/providers/v1/:namespace/:name/versions",
            get(versions),
        )
        .route(
            "/terraform/providers/v1/:namespace/:name/:version/download/:os/:arch",
            get(download),
        )
        .with_state(Arc::new(AppState {
            conf: conf::load_conf(opts.config).await?,
            client: Client::new(),
        }));
    Ok(axum::serve(tokio::net::TcpListener::bind("0.0.0.0:8080").await?, app).await?)
}
