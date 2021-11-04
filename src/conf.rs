use std::collections::HashMap;
use std::fs;
use std::error::Error;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigurationProvider {
    #[serde()]
    pub name: String,
    #[serde()]
    pub protocols: Vec<String>,
    #[serde()]
    pub version: ConfigurationVersion,
    #[serde()]
    pub binaries: Vec<ConfigurationBinary>,
    #[serde()]
    pub signature: ConfigurationSignature,
    #[serde()]
    pub artifact: ConfigurationArtifact,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigurationVersion {
    #[serde(alias = "type")]
    pub kind: String,
    #[serde()]
    pub uri: String,
    #[serde()]
    pub token: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigurationBinary {
    #[serde()]
    pub os: String,
    #[serde()]
    pub arch: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigurationArtifact {
    #[serde()]
    pub filename: String,
    #[serde()]
    pub download_url: String,
    #[serde()]
    pub shasums_url: String,
    #[serde()]
    pub shasums_signature_url: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigurationSignature {
    #[serde()]
    pub key_id: String,
    #[serde()]
    pub key_armor: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    #[serde()]
    pub providers: Vec<ConfigurationProvider>,
}
pub fn os_env_hashmap() -> HashMap<String, String> {
    env::vars_os()
        .filter(|v| v.0.to_owned().into_string().unwrap().to_lowercase().starts_with("terustry_"))
        .map(|v| (v.0.into_string().unwrap().to_lowercase(), v.1.into_string().unwrap()))
        .collect()

}
pub async fn load_conf(file: String) -> Result<Configuration, Box<dyn Error>> {
    Ok(serde_yaml::from_str(&fs::read_to_string(file)?)?)
}