#![no_std]
use loam_sdk::soroban_contract;
use loam_sdk_core_riff::{admin::Admin, CoreRiff};

pub mod error;
pub mod riff;

use error::Error;
use riff::{Calc, Calculator};

pub struct Contract;

impl CoreRiff for Contract {
    type Impl = Admin;
}

impl Calc for Contract {
    type Impl = Calculator;
}

soroban_contract!();
