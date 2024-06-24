#![allow(clippy::struct_excessive_bools)]
use crate::commands::build::env_toml;
use stellar_xdr::curr::Error as xdrError;
use soroban_cli::commands::NetworkRunnable;
use soroban_cli::utils::contract_hash;
use soroban_cli::{commands as cli, CommandParser};
use std::collections::BTreeMap as Map;
use std::fmt::Debug;
use std::hash::Hash;
use serde_json;

use super::env_toml::Network;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ValueEnum)]
pub enum LoamEnv {
    Development,
    Testing,
    Staging,
    Production,
}

#[derive(clap::Args, Debug, Clone, Copy)]
pub struct Args {
    #[arg(env = "LOAM_ENV", value_enum, default_value = "production")]
    pub env: LoamEnv,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    EnvironmentsToml(#[from] env_toml::Error),
    #[error("⛔ ️invalid network: must either specify a network name or both network_passphrase and rpc_url")]
    MalformedNetwork,
    #[error(transparent)]
    ParsingNetwork(#[from] cli::network::Error),
    #[error(transparent)]
    GeneratingKey(#[from] cli::keys::generate::Error),
    #[error("⛔ ️can only have one default account; marked as default: {0:?}")]
    OnlyOneDefaultAccount(Vec<String>),
    #[error("⛔ ️you need to provide at least one account, to use as the source account for contract deployment and other operations")]
    NeedAtLeastOneAccount,
    #[error("⛔ ️No contract named {0:?}")]
    BadContractName(String),
    #[error("⛔ ️Contract update not allowed in production for {0:?}")]
    ContractUpdateNotAllowed(String),
    #[error(transparent)]
    ContractInstall(#[from] cli::contract::install::Error),
    #[error(transparent)]
    ContractDeploy(#[from] cli::contract::deploy::wasm::Error),
    #[error(transparent)]
    ContractBindings(#[from] cli::contract::bindings::typescript::Error),
    #[error(transparent)]
    ContractFetch(#[from] cli::contract::fetch::Error),
    #[error(transparent)]
    ConfigLocator(#[from] cli::config::locator::Error),
    #[error(transparent)]
    Clap(#[from] clap::Error),
    #[error(transparent)]
    WasmHash(#[from] xdrError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl Args {
    pub async fn run(&self, workspace_root: &std::path::Path) -> Result<(), Error> {
        let Some(current_env) = env_toml::Environment::get(workspace_root, &self.loam_env())?
        else {
            return Ok(());
        };

        Self::add_network_to_env(&current_env.network)?;
        Self::handle_accounts(current_env.accounts.as_deref()).await?;
        self.handle_contracts(workspace_root, current_env.contracts.as_ref(), &current_env.network)
            .await?;

        Ok(())
    }

    fn loam_env(self) -> String {
        format!("{:?}", self.env).to_lowercase()
    }

    /// Parse the network settings from the environments.toml file and set `STELLAR_RPC_URL` and
    /// `STELLAR_NETWORK_PASSPHRASE`.
    ///
    /// We could set `STELLAR_NETWORK` instead, but when importing contracts, we want to hard-code
    /// the network passphrase. So if given a network name, we use soroban-cli to fetch the RPC url
    /// & passphrase for that named network, and still set the environment variables.
    fn add_network_to_env(network: &env_toml::Network) -> Result<(), Error> {
        match &network {
            Network {
                name: Some(name), ..
            } => {
                let cli::network::Network {
                    rpc_url,
                    network_passphrase,
                } = (cli::network::Args {
                    network: Some(name.clone()),
                    rpc_url: None,
                    network_passphrase: None,
                })
                .get(&cli::config::locator::Args {
                    global: false,
                    config_dir: None,
                })?;
                eprintln!("🌐 using {name} network");
                std::env::set_var("STELLAR_RPC_URL", rpc_url);
                std::env::set_var("STELLAR_NETWORK_PASSPHRASE", network_passphrase);
            }
            Network {
                rpc_url: Some(rpc_url),
                network_passphrase: Some(passphrase),
                ..
            } => {
                std::env::set_var("STELLAR_RPC_URL", rpc_url);
                std::env::set_var("STELLAR_NETWORK_PASSPHRASE", passphrase);
                eprintln!("🌐 using network at {rpc_url}");
            }
            _ => return Err(Error::MalformedNetwork),
        }

        Ok(())
    }
     
    fn get_contract_alias(&self, name: &str) -> Result<Option<String>, Error> {
        let config_dir = cli::config::locator::Args {
            global: false,
            config_dir: None,
        }.config_dir()?;
        let alias_file = config_dir.join("contract-ids").join(format!("{}.json", name));
        
        if alias_file.exists() {
            let content = std::fs::read_to_string(alias_file)?;
            let alias_data: serde_json::Value = serde_json::from_str(&content)?;
            let network_passphrase = std::env::var("STELLAR_NETWORK_PASSPHRASE")
                .expect("No STELLAR_NETWORK_PASSPHRASE environment variable set");
            
            if let Some(id) = alias_data["ids"].get(&network_passphrase) {
                Ok(Some(id.as_str().unwrap().to_string()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn contract_hash_matches(&self, contract_id: &str, hash: &str, network: &Network) -> Result<bool, Error> {
        let network_args = cli::network::Args {
            rpc_url: network.rpc_url.clone(),
            network_passphrase: network.network_passphrase.clone(),
            network: network.name.clone(),
        };
        let config_dir = cli::config::locator::Args {
            global: false,
            config_dir: None,
        };
        let hash_vec = cli::contract::fetch::Cmd {
            contract_id: contract_id.to_string(),
            out_file: None, 
            locator: config_dir,
            network: network_args,
        }
            .run_against_rpc_server(None, None)
            .await?;
        let ctrct_hash = contract_hash(&hash_vec)?;
        let c_hash_string = hex::encode(ctrct_hash).to_string();
        Ok(c_hash_string == hash)
    }

    fn save_contract_alias(&self, name: &str, contract_id: &str) -> Result<(), Error> {
        let config_dir = cli::config::locator::Args {
            global: false,
            config_dir: None,
        }.config_dir()?;
        let alias_dir = config_dir.join("contract-ids");
        std::fs::create_dir_all(&alias_dir)?;
        println!("{}", alias_dir.to_str().unwrap());
        
        let alias_file = alias_dir.join(format!("{}.json", name));
        let network_passphrase = std::env::var("STELLAR_NETWORK_PASSPHRASE")
            .expect("No STELLAR_NETWORK_PASSPHRASE environment variable set");
        
        let mut alias_data = if alias_file.exists() {
            let content = std::fs::read_to_string(&alias_file)?;
            serde_json::from_str(&content)?
        } else {
            serde_json::json!({ "ids": {} })
        };
    
        alias_data["ids"][&network_passphrase] = serde_json::Value::String(contract_id.to_string());
        
        let content = serde_json::to_string_pretty(&alias_data)?;
        std::fs::write(alias_file, content)?;
        
        Ok(())
    }

    fn write_contract_template(&self, workspace_root: &std::path::Path, name: &str, contract_id: &str) -> Result<(), Error> {
        let allow_http = if self.loam_env() == "development" {
            "\n  allowHttp: true,"
        } else {
            ""
        };
        let network = std::env::var("STELLAR_NETWORK_PASSPHRASE")
            .expect("No STELLAR_NETWORK_PASSPHRASE environment variable set");
        let template = format!(
            r#"import * as Client from '{name}';
    import {{ rpcUrl }} from './util';
    
    export default new Client.Client({{
      networkPassphrase: '{network}',
      contractId: '{contract_id}',
      rpcUrl,{allow_http}
      publicKey: undefined,
    }});
    "#
        );
        let path = workspace_root.join(format!("src/contracts/{name}.ts"));
        std::fs::write(path, template)?;
        Ok(())
    }

    async fn handle_accounts(accounts: Option<&[env_toml::Account]>) -> Result<(), Error> {
        let Some(accounts) = accounts else {
            return Err(Error::NeedAtLeastOneAccount);
        };

        let default_account_candidates = accounts
            .iter()
            .filter(|&account| account.default)
            .map(|account| account.name.clone())
            .collect::<Vec<_>>();

        let default_account = match (default_account_candidates.as_slice(), accounts) {
            ([], []) => return Err(Error::NeedAtLeastOneAccount),
            ([], [env_toml::Account { name, .. }, ..]) => name.clone(),
            ([candidate], _) => candidate.to_string(),
            _ => return Err(Error::OnlyOneDefaultAccount(default_account_candidates)),
        };

        for account in accounts {
            eprintln!("🔐 creating keys for {:?}", account.name);
            cli::keys::generate::Cmd::parse_arg_vec(&[&account.name])?
                .run()
                .await?;
        }

        std::env::set_var("STELLAR_ACCOUNT", &default_account);

        Ok(())
    }

    async fn handle_contracts(
        &self,
        workspace_root: &std::path::Path,
        contracts: Option<&Map<Box<str>, env_toml::Contract>>,
        network: &Network,
    ) -> Result<(), Error> {
        println!("Workspace root: {:?}", workspace_root);
        println!("Contracts: {:?}", contracts);
        let Some(contracts) = contracts else {
            return Ok(());
        };
        for (name, settings) in contracts {
            if settings.workspace {
                let wasm_path = workspace_root.join(format!("target/loam/{name}.wasm"));
                if !wasm_path.exists() {
                    return Err(Error::BadContractName(name.to_string()));
                }
                eprintln!("📲 installing {name:?} wasm bytecode on-chain...");
                let hash = cli::contract::install::Cmd::parse_arg_vec(&[
                    "--wasm",
                    wasm_path
                        .to_str()
                        .expect("we do not support non-utf8 paths"),
                ])?
                .run_against_rpc_server(None, None)
                .await?
                .into_result()
                .expect("no hash returned by 'contract install'")
                .to_string();
                eprintln!("    ↳ hash: {hash}");

                // Check if we have an alias saved for this contract
                let alias = self.get_contract_alias(name)?;
                println!("Debug: Contract name: {}, Alias: {:?}", name, alias);

                if let Some(contract_id) = alias {
                    // Check if the contract with that ID is using the current wasm hash
                    if self.contract_hash_matches(&contract_id, &hash, &network).await? {
                        eprintln!("✅ Contract {name:?} is up to date");
                        continue;
                    }

                    // For production environments, throw an error instead of updating
                    if self.loam_env() == "production" {
                        return Err(Error::ContractUpdateNotAllowed(name.to_string()));
                    }

                    eprintln!("🔄 Updating contract {name:?}");
                }

                eprintln!("🪞 instantiating {name:?} smart contract");
                let contract_id =
                    cli::contract::deploy::wasm::Cmd::parse_arg_vec(&["--alias", &name, "--wasm-hash", &hash])?
                        .run_against_rpc_server(None, None)
                        .await?
                        .into_result()
                        .expect("no contract id returned by 'contract deploy'");
                eprintln!("    ↳ contract_id: {contract_id}");

                // Save the alias for future use
                self.save_contract_alias(name, &contract_id)?;

                eprintln!("🎭 binding {name:?} contract");
                cli::contract::bindings::typescript::Cmd::parse_arg_vec(&[
                    "--contract-id",
                    &contract_id,
                    "--output-dir",
                    workspace_root
                        .join(format!("packages/{name}"))
                        .to_str()
                        .expect("we do not support non-utf8 paths"),
                    "--overwrite",
                ])?
                .run()
                .await?;

                eprintln!("🍽️ importing {:?} contract", name.clone());
                self.write_contract_template(workspace_root, name, &contract_id)?;
            };
        }

        Ok(())
    }

}
