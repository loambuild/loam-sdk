use std::collections::BTreeMap as Map;
use std::io;
use std::path::Path;
use soroban_cli::commands::network::Args as Network;
use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("⛔ ️parsing environments.toml: {0}")]
    ParsingToml(io::Error),
    #[error("⛔ ️no settings for current LOAM_ENV ({0:?}) found in environments.toml")]
    NoSettingsForCurrentEnv(String),
}

type Environments = Map<Box<str>, Environment>;

#[derive(Debug, Clone)]
pub struct Environment {
    pub accounts: Option<Vec<Account>>,
    pub network: Network,
    pub contracts: Option<Map<Box<str>, Contract>>,
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            accounts: Option<Vec<Account>>,
            network: NetworkHelper,
            contracts: Option<Map<Box<str>, Contract>>,
        }

        #[derive(Deserialize)]
        struct NetworkHelper {
            rpc_url: Option<String>,
            network_passphrase: Option<String>,
            network: Option<String>,
        }

        let helper = Helper::deserialize(deserializer)?;
        
        let network = Network {
            rpc_url: helper.network.rpc_url,
            network_passphrase: helper.network.network_passphrase,
            network: helper.network.network,
            ..Default::default() // This will set any other fields to their default values
        };

        Ok(Environment {
            accounts: helper.accounts,
            network,
            contracts: helper.contracts,
        })
    }
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
