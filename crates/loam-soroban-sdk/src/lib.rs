#![no_std]

pub use soroban_sdk::*;
pub mod into_key;

pub use into_key::IntoKey;

/// Trait for loading and setting a singleton type
pub trait Lazy: Sized {
    fn get_lazy() -> Option<Self>;

    fn set_lazy(self);
}

static mut ENV: Option<Env> = None;

pub fn set_env(env: Env) {
    unsafe { ENV = Some(env) };
}

pub fn env() -> &'static Env {
    unsafe { ENV.as_ref().unwrap() }
}

impl<T> Lazy for T
where
    T: IntoKey + TryFromVal<Env, Val> + IntoVal<Env, Val>,
{
    fn get_lazy() -> Option<Self> {
        env().storage().persistent().get(&Self::into_key())
    }

    fn set_lazy(self) {
        env().storage().persistent().set(&Self::into_key(), &self);
    }
}

pub use loam_sdk_macro::{IntoKey, Lazy};
