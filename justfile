set dotenv-load

export PATH := './target/bin:' + env_var('PATH')
export CONFIG_DIR := 'target/'
hash := `soroban contract install --wasm ./target/wasm32-unknown-unknown/contracts/example_status_message.wasm`

[private]
path:
    just --list

# build contracts
build:
    cargo build --package 'example*' --profile contracts --target wasm32-unknown-unknown

# Setup the project to use a pinned version of the CLI
setup:
    -cargo install --root ./target soroban-cli --debug --version 0.8.0 soroban-cli 


# Builds contracts. Deploys core riff and then redeploys to status message.
redeploy: build
    soroban config identity generate -d default
    soroban contract deploy --wasm ./target/wasm32-unknown-unknown/contracts/example_core.wasm
    soroban contract invoke -- owner_set --new_owner default
    soroban contract invoke -- --help
    soroban contract invoke -- redeploy --wasm_hash {{hash}}
    soroban contract invoke -- --help
    
    
