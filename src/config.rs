use anyhow::Context;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
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

        let database_url =
            std::env::var("DATABASE_URL")
            .context("DATABASE_URL must be set (see .env.example)")?;
        
        Ok(Self { bind_addr, database_url })
    }
}
