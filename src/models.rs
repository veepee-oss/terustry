use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct VersionPlatform {
    #[serde()]
    pub os: String,
    #[serde()]
    pub arch: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Version {
    #[serde()]
    pub version: String,
    #[serde()]
    pub protocols: Vec<String>,
    #[serde()]
    pub platforms: Vec<VersionPlatform>,
}

#[derive(Debug, Serialize, Clone)]
pub struct VersionsResponse {
    #[serde()]
    pub id: String,
    #[serde()]
    pub versions: Vec<Version>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadResponse {
    #[serde()]
    pub protocols: Vec<String>,
    #[serde()]
    pub os: String,
    #[serde()]
    pub arch: String,
    #[serde()]
    pub filename: String,
    #[serde()]
    pub download_url: String,
    #[serde()]
    pub shasums_url: String,
    #[serde()]
    pub shasums_signature_url: String,
    #[serde()]
    pub shasum: String,
    #[serde()]
    pub signing_keys: SigningKey,
}

#[derive(Debug, Serialize, Clone)]
pub struct SigningKey {
    #[serde()]
    pub gpg_public_keys: Vec<GpgPublicKey>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GpgPublicKey {
    #[serde()]
    pub key_id: String,
    #[serde()]
    pub ascii_armor: String,
}

#[derive(Serialize)]
pub struct Root {
    pub ok: bool,
}

#[derive(Serialize)]
pub struct WellKnown {
    pub providers_v1: String,
}
