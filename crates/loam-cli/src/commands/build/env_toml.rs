use std::collections::BTreeMap as Map;
use std::io;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("⛔ ️parsing environments.toml: {0}")]
    ParsingToml(io::Error),
    #[error("⛔ ️no settings for current LOAM_ENV ({0:?}) found in environments.toml")]
    NoSettingsForCurrentEnv(String),
}

type Environments = Map<Box<str>, Environment>;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Environment {
    pub accounts: Option<Vec<Account>>,
    pub network: Network,
    pub contracts: Option<Map<Box<str>, Contract>>,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    pub name: Option<String>,
    pub rpc_url: Option<String>,
    pub network_passphrase: Option<String>,
    // run_locally: Option<bool>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Account {
    pub name: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default: bool,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Contract {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub workspace: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init: Option<String>,
}

impl Environment {
    pub fn get(workspace_root: &Path, loam_env: &str) -> Result<Option<Environment>, Error> {
        let env_toml = workspace_root.join("environments.toml");

        if !env_toml.exists() {
            return Ok(None);
        }

        let toml_str = std::fs::read_to_string(env_toml).map_err(Error::ParsingToml)?;
        let mut parsed_toml: Environments = toml::from_str(&toml_str).unwrap();
        let current_env = parsed_toml.remove(loam_env);
        if current_env.is_none() {
            return Err(Error::NoSettingsForCurrentEnv(loam_env.to_string()));
        };
        Ok(current_env)
    }
}
