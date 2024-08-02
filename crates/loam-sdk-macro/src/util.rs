use std::{collections::BTreeMap, path::Path};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use soroban_sdk::xdr;
use syn::{File, ItemTrait, TraitItemFn};

/// Read a crate starting from a single file then parse into a file
pub fn parse_crate_as_file(path: &Path) -> Option<File> {
    syn_file_expand::read_crate(path)
        .map_err(|e| {
            println!("{e}");
            e
        })
        .ok()
        .map(|file| file.to_token_stream().to_string())
        .and_then(|file| syn::parse_file(&file).ok())
}

pub fn has_macro(attrs: &[syn::Attribute], macro_name: &str) -> bool {
    for attr in attrs {
        if format!("{:#?}", attr.path()).contains(macro_name) {
            return true;
        }
    }
    false
}

use syn::visit::Visit;

pub type Traits = BTreeMap<String, Vec<TokenStream>>;

#[derive(Default)]
pub struct TraitVisitor {
    pub traits: Traits,
}

impl TraitVisitor {
    pub fn find_traits_in_file(ast: &syn::File) -> Traits {
        let mut visitor = TraitVisitor::default();
        syn::visit::visit_file(&mut visitor, ast);
        visitor.traits
    }
}

impl<'ast> Visit<'ast> for TraitVisitor {
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if has_macro(&item.attrs, "subcontract") {
            self.traits
                .insert(item.ident.to_string(), generate_methods(item));
        }
    }
}

fn generate_methods(item: &ItemTrait) -> Vec<TokenStream> {
    item.items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(TraitItemFn { sig, attrs, .. }) = item {
                let name = &sig.ident;
                Some(generate_method(sig, attrs, name))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn generate_method(
    sig: &syn::Signature,
    attrs: &[syn::Attribute],
    name: &syn::Ident,
) -> TokenStream {
    let output = &sig.output;
    let inputs = sig.inputs.iter().skip(1);
    let args_without_self = crate::subcontract::get_args_without_self(&sig.inputs);
    quote! {
        #(#attrs)*
        pub fn #name(env: loam_sdk::soroban_sdk::Env, #(#inputs),*) #output {
            loam_sdk::soroban_sdk::set_env(env);
            Contract::#name(#(#args_without_self),*)
        }
    }
}

pub const LOCAL: &str = "Standalone Network ; February 2017";
pub const TESTNET: &str = "Test SDF Network ; September 2015";
pub const FUTURENET: &str = "Test SDF Future Network ; October 2022";
pub const MAINNET: &str = "Public Global Stellar Network ; September 2015";

///
/// Match network names to passphrases
///
///
pub fn network_passphrase(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "local" => LOCAL,
        "testnet" => TESTNET,
        "future" => FUTURENET,
        "main" => MAINNET,
        _ => "",
    }
}

pub fn generate_asset_id(
    asset: &str,
    network: &str,
) -> Result<stellar_strkey::Contract, xdr::Error> {
    use sha2::{Digest, Sha256};
    use xdr::WriteXdr;
    let asset = parse_asset(asset).unwrap();
    let network_passphrase = network_passphrase(network);
    let network_id = xdr::Hash(Sha256::digest(network_passphrase.as_bytes()).into());
    let preimage = xdr::HashIdPreimage::ContractId(xdr::HashIdPreimageContractId {
        network_id,
        contract_id_preimage: xdr::ContractIdPreimage::Asset(asset.clone()),
    });
    let preimage_xdr = preimage.to_xdr(xdr::Limits::none())?;
    Ok(stellar_strkey::Contract(
        Sha256::digest(preimage_xdr).into(),
    ))
}

pub fn parse_asset(str: &str) -> Result<xdr::Asset, xdr::Error> {
    if str == "native" {
        return Ok(xdr::Asset::Native);
    }
    let split: Vec<&str> = str.splitn(2, ':').collect();
    assert!(split.len() == 2, "invalid asset \"{str}\"");
    let code = split[0];
    let issuer = split[1];
    let re = regex::Regex::new("^[[:alnum:]]{1,12}$").expect("regex failed");
    assert!(re.is_match(code), "invalid asset \"{str}\"");
    if code.len() <= 4 {
        let mut asset_code: [u8; 4] = [0; 4];
        for (i, b) in code.as_bytes().iter().enumerate() {
            asset_code[i] = *b;
        }
        Ok(xdr::Asset::CreditAlphanum4(xdr::AlphaNum4 {
            asset_code: xdr::AssetCode4(asset_code),
            issuer: parse_account_id(issuer)?,
        }))
    } else {
        let mut asset_code: [u8; 12] = [0; 12];
        for (i, b) in code.as_bytes().iter().enumerate() {
            asset_code[i] = *b;
        }
        Ok(xdr::Asset::CreditAlphanum12(xdr::AlphaNum12 {
            asset_code: xdr::AssetCode12(asset_code),
            issuer: parse_account_id(issuer)?,
        }))
    }
}

pub fn parse_account_id(str: &str) -> Result<xdr::AccountId, stellar_strkey::DecodeError> {
    let pk_bytes = stellar_strkey::ed25519::PublicKey::from_string(str)?.0;
    Ok(xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(
        pk_bytes.into(),
    )))
}

// Generate the code to read the STELLAR_NETWORK environment variable
// and call the generate_asset_id function
pub fn parse_asset_literal(lit_str: &syn::LitStr, network: &str) -> TokenStream {
    let asset_code = lit_str.value();
    let asset_id = generate_asset_id(&asset_code, network).unwrap();
    let asset_id = stellar_strkey::Contract(asset_id.0).to_string();
    quote! {

            loam_sdk::soroban_sdk::token::Client::new(
                loam_sdk::soroban_sdk::env(),
                &loam_sdk::soroban_sdk::Address::from_string(
                        &loam_sdk::soroban_sdk::String::from_str(loam_sdk::soroban_sdk::env(), #asset_id,)
                )
            )

    }
}

#[allow(unused)]
pub(crate) fn equal_tokens(expected: &TokenStream, actual: &TokenStream) {
    assert_eq!(
        format_snippet(&expected.to_string()),
        format_snippet(&actual.to_string())
    );
}

pub(crate) fn p_e(e: std::io::Error) -> std::io::Error {
    eprintln!("{e:#?}");
    e
}

use std::io::{Read, Write};

/// Format the given snippet. The snippet is expected to be *complete* code.
/// When we cannot parse the given snippet, this function returns `None`.
#[allow(unused)]
pub(crate) fn format_snippet(snippet: &str) -> String {
    let mut child = std::process::Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(snippet.as_bytes())
        .map_err(p_e)
        .unwrap();
    child.wait().unwrap();
    let mut buf = String::new();
    child.stdout.unwrap().read_to_string(&mut buf).unwrap();
    println!("\n\n\n{buf}\n\n\n");
    buf
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_asset_id() {
        let asset_id = generate_asset_id("native", "local").unwrap();
        assert_eq!(
            asset_id.to_string(),
            "CDMLFMKMMD7MWZP3FKUBZPVHTUEDLSX4BYGYKH4GCESXYHS3IHQ4EIG4"
        );
    }

    #[test]
    fn test_generate_asset_id_code() {
        let asset_id = parse_asset_literal(
            &syn::LitStr::new("native", proc_macro2::Span::call_site()),
            "local",
        );
        equal_tokens(
            &asset_id,
            &quote! {
                loam_sdk::soroban_sdk::token::Client::new(loam_sdk::soroban_sdk::env(),
                    &loam_sdk::soroban_sdk::Address::from_string(
                    &loam_sdk::soroban_sdk::String::from_str( loam_sdk::soroban_sdk::env(), "CDMLFMKMMD7MWZP3FKUBZPVHTUEDLSX4BYGYKH4GCESXYHS3IHQ4EIG4"))
                )
            },
        );
    }
}
