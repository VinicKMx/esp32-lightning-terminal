//! Versioned wire protocol between terminals and the gateway.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, str::FromStr};
use terminal_models::{Bolt11, DeviceId, Invoice, InvoiceId, PaymentStatus, Sats, UnixTimestamp};
use thiserror::Error;

pub const API_VERSION: &str = "v1";
pub const HEADER_DEVICE_ID: &str = "x-terminal-device-id";
pub const HEADER_SIGNATURE: &str = "x-terminal-signature";
pub const HEADER_TIMESTAMP: &str = "x-terminal-timestamp";
pub const HEADER_IDEMPOTENCY_KEY: &str = "idempotency-key";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProtocolValidationError {
    #[error("{field} cannot be empty")]
    Empty { field: &'static str },

    #[error("{field} is too long")]
    TooLong { field: &'static str, max: usize },

    #[error("{field} contains invalid characters")]
    InvalidCharacters { field: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, ProtocolValidationError> {
        validate_token("idempotency_key", &value.into(), 128).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for IdempotencyKey {
    type Err = ProtocolValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for IdempotencyKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for IdempotencyKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(String);

impl RequestId {
    pub fn new(value: impl Into<String>) -> Result<Self, ProtocolValidationError> {
        validate_token("request_id", &value.into(), 128).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceAuth {
    pub device_id: DeviceId,
    pub timestamp: UnixTimestamp,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub amount_sats: Sats,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateInvoiceResponse {
    pub invoice_id: InvoiceId,
    pub amount_sats: Sats,
    pub bolt11: Bolt11,
    pub expires_at: UnixTimestamp,
}

impl From<&Invoice> for CreateInvoiceResponse {
    fn from(invoice: &Invoice) -> Self {
        Self {
            invoice_id: invoice.id.clone(),
            amount_sats: invoice.amount,
            bolt11: invoice.bolt11.clone(),
            expires_at: invoice.expires_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetInvoiceResponse {
    pub invoice_id: InvoiceId,
    pub status: PaymentStatus,
    pub amount_sats: Sats,
}

impl From<&Invoice> for GetInvoiceResponse {
    fn from(invoice: &Invoice) -> Self {
        Self {
            invoice_id: invoice.id.clone(),
            status: invoice.status,
            amount_sats: invoice.amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub request_id: RequestId,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>, request_id: RequestId) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            request_id,
        }
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<String, ProtocolValidationError> {
    if value.is_empty() {
        return Err(ProtocolValidationError::Empty { field });
    }

    if value.len() > max_len {
        return Err(ProtocolValidationError::TooLong {
            field,
            max: max_len,
        });
    }

    let valid = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'));

    if !valid {
        return Err(ProtocolValidationError::InvalidCharacters { field });
    }

    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_invoice_request_uses_protocol_field_names() -> Result<(), Box<dyn std::error::Error>>
    {
        let request = CreateInvoiceRequest {
            amount_sats: Sats::new(10_000)?,
        };

        let encoded = serde_json::to_string(&request)?;

        assert_eq!(encoded, r#"{"amount_sats":10000}"#);
        Ok(())
    }

    #[test]
    fn status_serializes_as_snake_case() -> Result<(), Box<dyn std::error::Error>> {
        let response = GetInvoiceResponse {
            invoice_id: InvoiceId::new("01JTEST")?,
            status: PaymentStatus::Paid,
            amount_sats: Sats::new(10_000)?,
        };

        let encoded = serde_json::to_string(&response)?;

        assert!(encoded.contains(r#""status":"paid""#));
        Ok(())
    }

    #[test]
    fn idempotency_key_rejects_spaces() {
        let result = IdempotencyKey::new("retry key");
        assert!(matches!(
            result,
            Err(ProtocolValidationError::InvalidCharacters {
                field: "idempotency_key"
            })
        ));
    }
}
