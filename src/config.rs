use std::{net::SocketAddr};
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
}

impl Config {
    /// Reads configuration from the environment.
    ///
    /// Everything is validated here so a bad value fails at startup rather
    /// than on the first request that happens to need it.
    pub fn from_env() -> anyhow::Result<Self> {
        let default_bind_addr = "127.0.0.1:8080";
        let bind_addr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| default_bind_addr.to_string())
            .parse()
            .context("BIND_ADDR must look like 127.0.0.1:8080")?;


        Ok(Self { bind_addr })
    }
}