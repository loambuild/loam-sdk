#![recursion_limit = "128"]
extern crate proc_macro;
use proc_macro::TokenStream;
use std::env;
use subcontract::derive_contract_impl;

use quote::quote;
use syn::Item;

mod contract;
mod subcontract;
mod util;

/// Generates a companion Trait which has a default type `Impl`, which implements this trait.
///
/// # Panics
///
/// This macro will panic if:
/// - The input `TokenStream` cannot be parsed into a valid Rust item.
/// - The `subcontract::generate` function fails to generate the companion trait.
///
#[proc_macro_attribute]
pub fn subcontract(_: TokenStream, item: TokenStream) -> TokenStream {
    let parsed: Item = syn::parse(item).unwrap();
    subcontract::generate(&parsed).into()
}

#[proc_macro_derive(IntoKey)]
pub fn into_key(item: TokenStream) -> TokenStream {
    syn::parse::<Item>(item)
        .and_then(subcontract::into_key::from_item)
        .map_or_else(|e| e.to_compile_error().into(), Into::into)
}

#[proc_macro_derive(Lazy)]
pub fn lazy(item: TokenStream) -> TokenStream {
    syn::parse::<Item>(item)
        .and_then(subcontract::lazy::from_item)
        .map_or_else(|e| e.to_compile_error().into(), Into::into)
}

pub(crate) fn manifest() -> std::path::PathBuf {
    std::path::PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("failed to finde cargo manifest"),
    )
    .join("Cargo.toml")
}

/// Generates a contract Client for a given contract.
/// It is expected that the name should be the same as the published contract or a contract in your current workspace.
///
/// # Panics
///
/// This function may panic in the following situations:
/// - If `loam_build::get_target_dir()` fails to retrieve the target directory
/// - If the input tokens cannot be parsed as a valid identifier
/// - If the directory path cannot be canonicalized
/// - If the canonical path cannot be converted to a string
#[proc_macro]
pub fn import_contract(tokens: TokenStream) -> TokenStream {
    let cargo_file = manifest();
    let mut dir = loam_build::get_target_dir(&cargo_file)
        .unwrap()
        .join(tokens.to_string());
    let name = syn::parse::<syn::Ident>(tokens).expect("The input must be a valid identifier");
    dir.set_extension("wasm");
    let binding = dir.canonicalize().unwrap();
    let file = binding.to_str().unwrap();
    quote! {
        mod #name {
            use loam_sdk::soroban_sdk;
            loam_sdk::soroban_sdk::contractimport!(file = #file);
        }
    }
    .into()
}

/// Generates a contract made up of subcontracts
/// ```ignore
/// #[derive_contract(Core(Admin), Postable(StatusMessage))]
/// pub struct Contract;
/// ```
/// Generates
///
/// ```ignore
/// pub struct Contract;
/// impl Postable for Contract {
///     type Impl = StatusMessage;
/// }
/// impl Core for Contract {
///     type Impl = Admin;
/// }
/// #[loam_sdk::soroban_sdk::contract]
/// struct SorobanContract__;
///
/// #[loam_sdk::soroban_sdk::contract]
/// impl SorobanContract__ {
///  // Postable and Core methods exposed
/// }
///
/// ```
///
/// # Panics
/// This function may panic if the input tokens cannot be parsed as a valid Rust item.
///
#[proc_macro_attribute]
pub fn derive_contract(args: TokenStream, item: TokenStream) -> TokenStream {
    let parsed: Item = syn::parse(item.clone()).expect("failed to parse Item");
    derive_contract_impl(proc_macro2::TokenStream::from(args), parsed).into()
}

/// Generates a contract Client for a given asset.
/// It is expected that the name of an asset, e.g. "native" or "USDC:G1...."
///
/// # Panics
///

#[proc_macro]
pub fn stellar_asset(input: TokenStream) -> TokenStream {
    // Parse the input as a string literal
    let input_str = syn::parse_macro_input!(input as syn::LitStr);
    let network = std::env::var("STELLAR_NETWORK").unwrap_or_else(|_| "local".to_owned());
    let asset = util::parse_asset_literal(&input_str, &network);

    // Return the generated code as a TokenStream
    asset.into()
}
