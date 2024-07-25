use std::sync::Arc;
use std::time::Duration;

use crate::conf::{Configuration, ConfigurationProvider};
use crate::errors::AppError;
use crate::models::*;

use anyhow::anyhow;
use artifacts::ArtifactClient;
use axum::extract::{Path, State};
use axum::routing::get;
use versions::VersionClient;

use axum::{Json, Router};
use clap::Parser;
use reqwest::{Client, ClientBuilder};
use tracing::info;

mod artifacts;
mod conf;
mod errors;
mod github;
mod gitlab;
mod models;
// NOTE: What is the difference between `use crate` and `mod`?
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
    let version_client = &data.version_client;
    Ok(Json(VersionsResponse {
        id: provider.name.clone(),
        versions: version_client
            .get_versions(&data.client, &provider)
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

async fn invalidate_versions(
    State(data): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<(), AppError> {
    let provider = get_provider_conf(&data.conf, namespace, name)?;
    let version_client = &data.version_client;
    version_client.invalidate(&provider).await;
    Ok(())
}

async fn download(
    State(data): State<Arc<AppState>>,
    Path((namespace, name, version, os, arch)): Path<(String, String, String, String, String)>,
) -> Result<Json<DownloadResponse>, AppError> {
    info!("download {}/{}/{} for {}/{}", namespace, name, version, os, arch);
    let artifact_client = &data.artifact_client;
    Ok(Json(
        artifact_client.get(
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

async fn invalidate_artifact(
    State(data): State<Arc<AppState>>,
    Path((namespace, name, version, os, arch)): Path<(String, String, String, String, String)>,
) -> Result<(), AppError> {
    let provider = get_provider_conf(&data.conf, namespace, name)?;
    let artifact_client = &data.artifact_client;
    artifact_client.invalidate(
        provider,
        version,
        os,
        arch
    ).await;
    Ok(())
}

struct AppState {
    conf: Configuration,
    client: Client,
    version_client: VersionClient,
    artifact_client: ArtifactClient,
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
    let client_builder = ClientBuilder::new().timeout(Duration::new(10, 0));
    let version_client = VersionClient::new();
    let artifact_client = ArtifactClient::new();
    let app = Router::new()
        .route("/", get(root))
        .route("/.well-known/terraform.json", get(well_known))
        .route(
            "/terraform/providers/v1/:namespace/:name/versions",
            get(versions),
        )
        .route(
            "/terraform/providers/v1/:namespace/:name/invalidate",
            get(invalidate_versions),
        )
        .route(
            "/terraform/providers/v1/:namespace/:name/:version/download/:os/:arch",
            get(download),
        )
        .route(
            "/terraform/providers/v1/:namespace/:name/:version/invalidate/:os/:arch",
            get(invalidate_artifact),
        )
        .with_state(Arc::new(AppState {
            conf: conf::load_conf(opts.config).await?,
            client: client_builder.build()?,
            version_client,
            artifact_client,
        }));
    Ok(axum::serve(tokio::net::TcpListener::bind("0.0.0.0:8080").await?, app).await?)
}
