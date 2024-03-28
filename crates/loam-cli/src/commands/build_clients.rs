#![allow(clippy::struct_excessive_bools)]
use clap::Parser;
use std::fmt::Debug;

#[derive(Parser, Debug, Clone)]
pub struct Cmd {
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
}

impl Cmd {
    pub fn run(&self) -> Result<(), Error> {
        println!("No environments.toml; nothing to do");
        Ok(())
    }
}
