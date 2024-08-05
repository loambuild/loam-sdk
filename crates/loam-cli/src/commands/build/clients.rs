#![allow(clippy::struct_excessive_bools)]
use crate::commands::build::env_toml;
use regex::Regex;
use serde_json;
use shlex::split;
use soroban_cli::commands::NetworkRunnable;
use soroban_cli::utils::contract_hash;
use soroban_cli::{commands as cli, CommandParser};
use std::collections::BTreeMap as Map;
use std::fmt::Debug;
use std::hash::Hash;
use std::process::Command;
use stellar_strkey;
use stellar_xdr::curr::Error as xdrError;

use super::env_toml::Network;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ValueEnum)]
pub enum LoamEnv {
    Development,
    Testing,
    Staging,
    Production,
}

impl std::fmt::Display for LoamEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}

#[derive(clap::Args, Debug, Clone, Copy)]
pub struct Args {
    #[arg(env = "LOAM_ENV", value_enum)]
    pub env: Option<LoamEnv>,
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
    #[error(
        "⛔ ️Contract must be identified by its ID in production or staging. Invalid ID: {0:?}"
    )]
    InvalidContractID(String),
    #[error("⛔ ️Unable to parse init script: {0:?}")]
    InitParseFailure(String),
    #[error("⛔ ️Failed to execute subcommand: {0:?}\n{1:?}")]
    SubCommandExecutionFailure(String, String),
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
    ContractInvoke(#[from] cli::contract::invoke::Error),
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
    pub async fn run(
        &self,
        workspace_root: &std::path::Path,
        package_names: Vec<String>,
    ) -> Result<(), Error> {
        let Some(current_env) =
            env_toml::Environment::get(workspace_root, &self.loam_env(LoamEnv::Production))?
        else {
            return Ok(());
        };

        Self::add_network_to_env(&current_env.network)?;
        Self::handle_accounts(current_env.accounts.as_deref()).await?;
        self.handle_contracts(
            workspace_root,
            current_env.contracts.as_ref(),
            package_names,
            &current_env.network,
        )
        .await?;

        Ok(())
    }

    fn loam_env(self, default: LoamEnv) -> String {
        self.env.unwrap_or(default).to_string().to_lowercase()
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

    fn get_network_args(network: &Network) -> cli::network::Args {
        cli::network::Args {
            rpc_url: network.rpc_url.clone(),
            network_passphrase: network.network_passphrase.clone(),
            network: network.name.clone(),
        }
    }

    fn get_config_locator() -> cli::config::locator::Args {
        cli::config::locator::Args {
            global: false,
            config_dir: None,
        }
    }

    fn get_contract_alias(name: &str) -> Result<Option<String>, cli::config::locator::Error> {
        let config_dir = Self::get_config_locator();
        let network_passphrase = std::env::var("STELLAR_NETWORK_PASSPHRASE")
            .expect("No STELLAR_NETWORK_PASSPHRASE environment variable set");
        config_dir.get_contract_id(name, &network_passphrase)
    }

    async fn contract_hash_matches(
        &self,
        contract_id: &str,
        hash: &str,
        network: &Network,
    ) -> Result<bool, Error> {
        let result = cli::contract::fetch::Cmd {
            contract_id: contract_id.to_string(),
            out_file: None,
            locator: Self::get_config_locator(),
            network: Self::get_network_args(network),
        }
        .run_against_rpc_server(None, None)
        .await;

        match result {
            Ok(result) => {
                let ctrct_hash = contract_hash(&result)?;
                Ok(hex::encode(ctrct_hash) == hash)
            }
            Err(e) => {
                if e.to_string().contains("Contract not found") {
                    Ok(false)
                } else {
                    Err(Error::ContractFetch(e))
                }
            }
        }
    }

    fn save_contract_alias(
        name: &str,
        contract_id: &str,
        network: &Network,
    ) -> Result<(), cli::config::locator::Error> {
        let config_dir = Self::get_config_locator();
        let passphrase = network
            .network_passphrase
            .clone()
            .expect("You must set a network passphrase.");
        config_dir.save_contract_id(&passphrase, contract_id, name)
    }

    fn write_contract_template(
        self,
        workspace_root: &std::path::Path,
        name: &str,
        contract_id: &str,
    ) -> Result<(), Error> {
        let allow_http = if self.loam_env(LoamEnv::Production) == "development" {
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

    async fn account_exists(account_name: &str) -> Result<bool, Error> {
        // TODO: this is a workaround until generate is changed to not overwrite accounts
        Ok(cli::keys::fund::Cmd::parse_arg_vec(&[account_name])?
            .run()
            .await
            .is_ok())
    }

    async fn generate_contract_bindings(
        self,
        workspace_root: &std::path::Path,
        name: &str,
        contract_id: &str,
    ) -> Result<(), Error> {
        eprintln!("🎭 binding {name:?} contract");
        cli::contract::bindings::typescript::Cmd::parse_arg_vec(&[
            "--contract-id",
            contract_id,
            "--output-dir",
            workspace_root
                .join(format!("packages/{name}"))
                .to_str()
                .expect("we do not support non-utf8 paths"),
            "--overwrite",
        ])?
        .run()
        .await?;

        eprintln!("🍽️ importing {name:?} contract");
        self.write_contract_template(workspace_root, name, contract_id)?;

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
            if Self::account_exists(&account.name).await? {
                eprintln!(
                    "ℹ️ account {:?} already exists, skipping key creation",
                    account.name
                );
            } else {
                eprintln!("🔐 creating keys for {:?}", account.name);
                cli::keys::generate::Cmd::parse_arg_vec(&[&account.name])?
                    .run()
                    .await?;
            }
        }

        std::env::set_var("STELLAR_ACCOUNT", &default_account);

        Ok(())
    }

    async fn handle_contracts(
        self,
        workspace_root: &std::path::Path,
        contracts: Option<&Map<Box<str>, env_toml::Contract>>,
        package_names: Vec<String>,
        network: &Network,
    ) -> Result<(), Error> {
        if package_names.is_empty() {
            return Ok(());
        }
        let env = self.loam_env(LoamEnv::Production);
        if env == "production" || env == "staging" {
            if let Some(contracts) = contracts {
                for (name, _) in contracts.iter().filter(|(_, settings)| settings.client) {
                    // ensure contract names are valid contract IDs
                    if stellar_strkey::Contract::from_string(name.as_ref()).is_err() {
                        return Err(Error::InvalidContractID(name.to_string()));
                    }
                    self.generate_contract_bindings(workspace_root, name, name)
                        .await?;
                }
            }
            return Ok(());
        }

        // ensure contract names are valid
        if let Some(contracts) = contracts {
            for (name, _) in contracts.iter().filter(|(_, settings)| settings.client) {
                let wasm_path = workspace_root.join(format!("target/loam/{name}.wasm"));
                if !wasm_path.exists() {
                    return Err(Error::BadContractName(name.to_string()));
                }
            }
        }
        for name in package_names {
            let settings = match contracts {
                Some(contracts) => contracts.get(&name as &str),
                None => None,
            };
            // Skip only if contract is found and its `client` setting is false
            if let Some(c) = settings {
                if !c.client {
                    continue;
                }
            }
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
            let alias = Self::get_contract_alias(&name)?;
            if let Some(contract_id) = alias {
                match self
                    .contract_hash_matches(&contract_id, &hash, network)
                    .await
                {
                    Ok(true) => {
                        eprintln!("✅ Contract {name:?} is up to date");
                        continue;
                    }
                    Ok(false) => eprintln!("🔄 Updating contract {name:?}"),
                    Err(e) => return Err(e),
                }
            }

            eprintln!("🪞 instantiating {name:?} smart contract");
            let contract_id = cli::contract::deploy::wasm::Cmd::parse_arg_vec(&[
                "--alias",
                &name,
                "--wasm-hash",
                &hash,
            ])?
            .run_against_rpc_server(None, None)
            .await?
            .into_result()
            .expect("no contract id returned by 'contract deploy'");
            eprintln!("    ↳ contract_id: {contract_id}");

            // Save the alias for future use
            Self::save_contract_alias(&name, &contract_id, network)?;

            // Run init script if we're in development or test environment
            if env == "development" || env == "testing" {
                if let Some(settings) = settings {
                    if let Some(init_script) = &settings.init {
                        eprintln!("🚀 Running initialization script for {name:?}");
                        self.run_init_script(&name, &contract_id, init_script)
                            .await?;
                    }
                }
            }
            self.generate_contract_bindings(workspace_root, &name, &contract_id)
                .await?;
        }

        Ok(())
    }

    fn resolve_line(re: &Regex, line: &str, shell: &str, flag: &str) -> Result<String, Error> {
        let mut result = String::new();
        let mut last_match = 0;
        for cap in re.captures_iter(line) {
            let whole_match = cap.get(0).unwrap();
            result.push_str(&line[last_match..whole_match.start()]);
            let cmd = &cap[1];
            let output = Self::execute_subcommand(shell, flag, cmd)?;
            result.push_str(&output);
            last_match = whole_match.end();
        }
        result.push_str(&line[last_match..]);
        Ok(result)
    }

    fn execute_subcommand(shell: &str, flag: &str, cmd: &str) -> Result<String, Error> {
        match Command::new(shell).arg(flag).arg(cmd).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                if output.status.success() {
                    Ok(stdout)
                } else {
                    Err(Error::SubCommandExecutionFailure(cmd.to_string(), stderr))
                }
            }
            Err(e) => Err(Error::SubCommandExecutionFailure(
                cmd.to_string(),
                e.to_string(),
            )),
        }
    }

    async fn run_init_script(
        &self,
        name: &str,
        contract_id: &str,
        init_script: &str,
    ) -> Result<(), Error> {
        let re = Regex::new(r"\$\((.*?)\)").expect("Invalid regex pattern");

        let (shell, flag) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        for line in init_script.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // resolve any $() patterns
            let resolved_line = Self::resolve_line(&re, line, shell, flag)?;
            let parts = split(&resolved_line)
                .ok_or_else(|| Error::InitParseFailure(resolved_line.to_string()))?;
            let (source_account, command_parts): (Vec<_>, Vec<_>) = parts
                .iter()
                .partition(|&part| part.starts_with("STELLAR_ACCOUNT="));

            let mut args = vec!["--id", contract_id];
            if let Some(account) = source_account.first() {
                let account = account.strip_prefix("STELLAR_ACCOUNT=").unwrap();
                args.extend_from_slice(&["--source-account", account]);
            }
            args.extend_from_slice(&["--"]);
            args.extend(command_parts.iter().map(|s| s.as_str()));

            eprintln!("  ↳ Executing: soroban contract invoke {}", args.join(" "));
            let result = cli::contract::invoke::Cmd::parse_arg_vec(&args)?
                .run_against_rpc_server(None, None)
                .await?;
            eprintln!("  ↳ Result: {result:?}");
        }
        eprintln!("✅ Initialization script for {name:?} completed successfully");
        Ok(())
    }
}
