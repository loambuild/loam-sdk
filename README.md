# Loam SDK

A Software Development Kit (SDK) and build tool for writing smart contracts in Rust on Wasm blockchains.

Currently, the focus is on the Soroban VM, but the same ideas apply to other VMs.

## Table of Contents

- [Loam SDK](#loam-sdk)
  - [Table of Contents](#table-of-contents)
  - [Getting Started](#getting-started)
    - [Installation](#installation)
    - [Setup](#setup)
    - [Redeploy](#redeploy)
  - [Contract Riffs](#contract-riffs)
    - [Creating Contract Riffs](#creating-contract-riffs)
    - [External API](#external-api)
  - [CoreRiff](#coreriff)
    - [Using the CoreRiff](#using-the-coreriff)

## Getting Started

### Installation

To install `just`, run the following command:

```bash
cargo install just
```

### Setup

To set up the environment, run:

```bash
just setup
```

### Redeploy

To see redeployment in action, use:

```bash
just redeploy
```

## Contract Riffs

A contract riff (or mixin) is a type that implements the `IntoKey` trait, which is used for lazily loading and storing the type.

### Creating Contract Riffs

Here's an example of how to create a contract riff:

```rust
#[contracttype]
#[derive(IntoKey)]
pub struct Messages(Map<Address, String>);
```

This generates the following implementation:

```rust
impl IntoKey for Messages {
    type Key = IntoVal<Env, RawVal>;
    fn into_key() -> Self::Key {
      String::from_slice("messages")
    }
```

### External API

You can also create and implement external APIs for contract riffs:

```rust
#[riff]
pub trait IsPostable {
    fn messages_get(&self, author: Address) -> Option<String>;
    fn messages_set(&mut self, author: Address, text: String);
}
```

## CoreRiff

The `CoreRiff` trait provides the minimum logic needed for a contract to be redeployable. A contract should be able to be redeployed to another wasm binary that can also be redeployed. Redeployment requires the contract to have an admin, as it would be undesirable for any account to redeploy the contract.

### Using the CoreRiff

To use the core riff, create a `Contract` structure and implement the `CoreRiff` for it. The `Contract` will be redeployable and will be able to implement other Riffs.

```rust
use loam_sdk::{soroban_contract, soroban_sdk};
use loam_sdk_core_riff::{admin::Admin, CoreRiff};

pub struct Contract;

impl CoreRiff for Contract {
    type Impl = Admin;
}

soroban_contract!();
```

This code generates the following implementation:

```rust
struct SorobanContract;

#[contractimpl]
impl SorobanContract {
     pub fn admin_set(env: Env, admin: Address) {
        set_env(env);
        Contract::admin_set(admin);
    }
    pub fn admin_get(env: Env) -> Option<Address> {
        set_env(env);
        Contract::admin_get()
    }
    pub fn redeploy(env: Env, wasm_hash: BytesN<32>) {
        set_env(env);
        Contract::redeploy(wasm_hash);
    }
    // Riff methods would be inserted here.
    // Contract must implement all Riffs and is the proxy for the contract calls.
    // This is because the Riffs have default implementations which call the associated type
}
```

By specifying the associated `Impl` type for `CoreRiff`, you enable the default `Admin` methods to be used (`admin_set`, `admin_get`, `redeploy`). However, you can also provide a different implementation if needed by replacing `Admin` with a different struct/enum that also implements [IsCoreRiff](https://github.com/loambuild/loam-sdk/blob/5473bb20fb3c818e7c30652fadf66647760a408d/crates/loam-core/src/admin.rs#L41-L51).

Notice that the generated code calls `Contract::redeploy` and other methods. This ensures that the `Contract` type is redeployable, while also allowing for extensions, as `Contract` can overwrite the default methods.
