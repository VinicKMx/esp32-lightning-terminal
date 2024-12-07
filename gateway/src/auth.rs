use axum::http::{HeaderMap, Method};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use terminal_models::{DeviceId, UnixTimestamp};
use terminal_protocol::{HEADER_DEVICE_ID, HEADER_SIGNATURE, HEADER_TIMESTAMP};
use thiserror::Error;

use crate::config::GatewayConfig;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing header {0}")]
    MissingHeader(&'static str),

    #[error("invalid header {0}")]
    InvalidHeader(&'static str),

    #[error("unknown device")]
    UnknownDevice,

    #[error("request timestamp is outside the accepted clock skew")]
    TimestampOutsideSkew,

    #[error("invalid request signature")]
    InvalidSignature,
}

pub fn sign_request(
    secret: &str,
    method: &Method,
    path: &str,
    timestamp: UnixTimestamp,
    body: &[u8],
) -> Result<String, AuthError> {
    let payload = signing_payload(method, path, timestamp, body);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| AuthError::InvalidSignature)?;
    mac.update(payload.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn verify_request(
    config: &GatewayConfig,
    headers: &HeaderMap,
    method: &Method,
    path: &str,
    body: &[u8],
) -> Result<DeviceId, AuthError> {
    let device_id = header_value(headers, HEADER_DEVICE_ID)?;
    let signature = header_value(headers, HEADER_SIGNATURE)?;
    let timestamp = header_value(headers, HEADER_TIMESTAMP)?;

    let device_id =
        DeviceId::new(device_id).map_err(|_| AuthError::InvalidHeader(HEADER_DEVICE_ID))?;
    if device_id != config.device_id {
        return Err(AuthError::UnknownDevice);
    }

    let timestamp = timestamp
        .parse::<u64>()
        .map(UnixTimestamp::from_secs)
        .map_err(|_| AuthError::InvalidHeader(HEADER_TIMESTAMP))?;
    validate_timestamp(timestamp, config.auth_max_skew_seconds)?;

    let payload = signing_payload(method, path, timestamp, body);
    let provided_signature =
        hex::decode(signature).map_err(|_| AuthError::InvalidHeader(HEADER_SIGNATURE))?;
    let mut mac = HmacSha256::new_from_slice(config.device_secret.as_bytes())
        .map_err(|_| AuthError::InvalidSignature)?;
    mac.update(payload.as_bytes());
    mac.verify_slice(&provided_signature)
        .map_err(|_| AuthError::InvalidSignature)?;

    Ok(device_id)
}

fn signing_payload(method: &Method, path: &str, timestamp: UnixTimestamp, body: &[u8]) -> String {
    let body_hash = Sha256::digest(body);
    format!(
        "{}\n{}\n{}\n{}",
        method.as_str(),
        path,
        timestamp.as_secs(),
        hex::encode(body_hash)
    )
}

fn header_value<'a>(headers: &'a HeaderMap, name: &'static str) -> Result<&'a str, AuthError> {
    headers
        .get(name)
        .ok_or(AuthError::MissingHeader(name))?
        .to_str()
        .map_err(|_| AuthError::InvalidHeader(name))
}

fn validate_timestamp(timestamp: UnixTimestamp, max_skew_seconds: u64) -> Result<(), AuthError> {
    let now = UnixTimestamp::now().as_secs();
    let request_time = timestamp.as_secs();
    let delta = now.abs_diff(request_time);

    if delta > max_skew_seconds {
        Err(AuthError::TimestampOutsideSkew)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn signature_roundtrip_verifies() -> Result<(), Box<dyn std::error::Error>> {
        let config = GatewayConfig {
            bind_addr: "127.0.0.1:3000".parse()?,
            device_id: DeviceId::new("dev-terminal-001")?,
            device_secret: "secret".to_owned(),
            invoice_ttl_seconds: 900,
            auth_max_skew_seconds: 300,
        };
        let method = Method::POST;
        let path = "/v1/invoices";
        let timestamp = UnixTimestamp::now();
        let body = br#"{"amount_sats":10000}"#;
        let signature = sign_request(&config.device_secret, &method, path, timestamp, body)?;
        let mut headers = HeaderMap::new();
        headers.insert(
            HEADER_DEVICE_ID,
            HeaderValue::from_static("dev-terminal-001"),
        );
        headers.insert(
            HEADER_TIMESTAMP,
            HeaderValue::from_str(&timestamp.to_string())?,
        );
        headers.insert(HEADER_SIGNATURE, HeaderValue::from_str(&signature)?);

        let verified = verify_request(&config, &headers, &method, path, body)?;

        assert_eq!(verified, config.device_id);
        Ok(())
    }
}
