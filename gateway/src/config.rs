use std::{env, net::SocketAddr};
use terminal_models::DeviceId;
use thiserror::Error;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_DEVICE_ID: &str = "dev-terminal-001";
const DEFAULT_DEVICE_SECRET: &str = "change-me-development-secret";
const DEFAULT_INVOICE_TTL_SECONDS: u64 = 900;
const DEFAULT_AUTH_MAX_SKEW_SECONDS: u64 = 300;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid bind address: {0}")]
    InvalidBindAddress(#[from] std::net::AddrParseError),

    #[error("invalid device id: {0}")]
    InvalidDeviceId(#[from] terminal_models::ValidationError),

    #[error("invalid integer in {name}: {source}")]
    InvalidInteger {
        name: &'static str,
        source: std::num::ParseIntError,
    },
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub bind_addr: SocketAddr,
    pub device_id: DeviceId,
    pub device_secret: String,
    pub invoice_ttl_seconds: u64,
    pub auth_max_skew_seconds: u64,
}

impl GatewayConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = env::var("GATEWAY_BIND_ADDR")
            .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned())
            .parse()?;
        let device_id = DeviceId::new(
            env::var("GATEWAY_DEVICE_ID").unwrap_or_else(|_| DEFAULT_DEVICE_ID.to_owned()),
        )?;
        let device_secret =
            env::var("GATEWAY_DEVICE_SECRET").unwrap_or_else(|_| DEFAULT_DEVICE_SECRET.to_owned());
        let invoice_ttl_seconds =
            parse_u64_env("GATEWAY_INVOICE_TTL_SECONDS", DEFAULT_INVOICE_TTL_SECONDS)?;
        let auth_max_skew_seconds = parse_u64_env(
            "GATEWAY_AUTH_MAX_SKEW_SECONDS",
            DEFAULT_AUTH_MAX_SKEW_SECONDS,
        )?;

        Ok(Self {
            bind_addr,
            device_id,
            device_secret,
            invoice_ttl_seconds,
            auth_max_skew_seconds,
        })
    }
}

fn parse_u64_env(name: &'static str, default: u64) -> Result<u64, ConfigError> {
    env::var(name)
        .map(|value| {
            value
                .parse()
                .map_err(|source| ConfigError::InvalidInteger { name, source })
        })
        .unwrap_or(Ok(default))
}
