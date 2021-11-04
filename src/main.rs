use actix_web::{App, error, Error, get, HttpResponse, HttpServer, Responder, web};
use actix_web::error::ErrorNotFound;
use crate::conf::{Configuration, ConfigurationProvider};
use crate::models::*;
use clap::Parser;

mod gitlab;
mod models;
mod conf;
mod artifacts;
mod github;

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().content_type("application/json").body(r#"{"ok": true}"#)
}

#[get("/.well-known/terraform.json")]
async fn well_known() -> impl Responder {
    HttpResponse::Ok().content_type("application/json").body(r#"{"providers.v1": "/terraform/providers/v1/"}"#)
}

fn get_provider_conf(data: web::Data<AppState>, namespace: String, name: String) -> Option<ConfigurationProvider> {
    data.conf.clone().providers.into_iter().find(|p| p.name == format!("{}/{}", namespace, name))
}

#[get("/terraform/providers/v1/{namespace}/{name}/versions")]
async fn versions(data: web::Data<AppState>, web::Path((namespace, name)): web::Path<(String, String)>) -> Result<HttpResponse, Error> {
    match get_provider_conf(data, namespace, name) {
        Some(provider) => {
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(
                    serde_json::to_string::<VersionsResponse>(
                        &VersionsResponse {
                            id: provider.name.clone(),
                            versions: match provider.version.kind.as_str() {
                                "gitlab" => gitlab::versions(&provider).await.map_err(|e| error::ErrorInternalServerError(e.to_string()))?,
                                "github" => github::versions(&provider).await.map_err(|e| error::ErrorInternalServerError(e.to_string()))?,
                                _ => return Err(ErrorNotFound(String::from("Provider not found"))),
                            }.iter().map(|version| {
                                Version {
                                    version: version.to_owned().trim_start_matches("v").to_string(),
                                    protocols: provider.protocols.to_owned(),
                                    platforms: provider.binaries.to_owned().into_iter().map(|binary| {
                                        VersionPlatform { os: binary.os, arch: binary.arch }
                                    }).collect::<Vec<VersionPlatform>>(),
                                }
                            }).collect()
                        }
                    )?))
        }
        None => Err(ErrorNotFound(String::from("Provider not found"))),
    }
}

#[get("/terraform/providers/v1/{namespace}/{name}/{version}/download/{os}/{arch}")]
async fn download(data: web::Data<AppState>, web::Path((namespace, name, version, os, arch)): web::Path<(String, String, String, String, String)>) -> Result<HttpResponse, Error> {
    match get_provider_conf(data, namespace, name) {
        Some(provider) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string::<DownloadResponse>(
                &artifacts::get(provider, version, os, arch).await.map_err(|e| error::ErrorInternalServerError(e.to_string()))?
            )?)),
        None => Err(ErrorNotFound(String::from("Provider not found"))),
    }
}

struct AppState {
    conf: Configuration,
}

#[derive(Parser)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(short, long, default_value = "/etc/terustry.yml")]
    config: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    match conf::load_conf(opts.config).await {
        Ok(conf) => {
            HttpServer::new(move || {
                App::new()
                    .data(AppState {
                        conf: conf.clone(),
                    })
                    .service(root)
                    .service(well_known)
                    .service(versions)
                    .service(download)
            })
                .bind("0.0.0.0:8080")?.run().await
        }
        Err(e) => {
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Fail to load configuration {}", e)))
        }
    }
}