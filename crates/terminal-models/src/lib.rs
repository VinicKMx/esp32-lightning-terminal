//! Shared domain model for Lightning payment terminals.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{
    fmt,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

const MAX_BITCOIN_SUPPLY_SATS: u64 = 21_000_000 * 100_000_000;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("{field} cannot be empty")]
    Empty { field: &'static str },

    #[error("{field} is too long")]
    TooLong { field: &'static str, max: usize },

    #[error("{field} contains invalid characters")]
    InvalidCharacters { field: &'static str },

    #[error("{field} must be greater than zero")]
    ZeroAmount { field: &'static str },

    #[error("{field} exceeds the maximum Bitcoin supply")]
    AmountTooLarge { field: &'static str },

    #[error("{field} is not a valid BOLT11 invoice")]
    InvalidBolt11 { field: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sats(u64);

impl Sats {
    pub fn new(value: u64) -> Result<Self, ValidationError> {
        if value == 0 {
            return Err(ValidationError::ZeroAmount {
                field: "amount_sats",
            });
        }

        if value > MAX_BITCOIN_SUPPLY_SATS {
            return Err(ValidationError::AmountTooLarge {
                field: "amount_sats",
            });
        }

        Ok(Self(value))
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Sats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u64> for Sats {
    type Error = ValidationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for Sats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for Sats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixTimestamp(u64);

impl UnixTimestamp {
    pub fn from_secs(value: u64) -> Self {
        Self(value)
    }

    pub fn now() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        Self(seconds)
    }

    pub fn now_plus_seconds(seconds: u64) -> Self {
        let now = Self::now().as_secs();
        Self(now.saturating_add(seconds))
    }

    pub fn as_secs(self) -> u64 {
        self.0
    }
}

impl fmt::Display for UnixTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        validate_identifier("device_id", &value.into(), 128).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for DeviceId {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for DeviceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvoiceId(String);

impl InvoiceId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        validate_identifier("invoice_id", &value.into(), 128).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InvoiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for InvoiceId {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for InvoiceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for InvoiceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bolt11(String);

impl Bolt11 {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ValidationError::Empty { field: "bolt11" });
        }

        if value.len() > 4096 {
            return Err(ValidationError::TooLong {
                field: "bolt11",
                max: 4096,
            });
        }

        if !value.is_ascii() || value.chars().any(char::is_whitespace) {
            return Err(ValidationError::InvalidCharacters { field: "bolt11" });
        }

        if !value.to_ascii_lowercase().starts_with("ln") {
            return Err(ValidationError::InvalidBolt11 { field: "bolt11" });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Bolt11 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Bolt11 {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for Bolt11 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Bolt11 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Paid,
    Expired,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invoice {
    pub id: InvoiceId,
    pub amount: Sats,
    pub bolt11: Bolt11,
    pub expires_at: UnixTimestamp,
    pub status: PaymentStatus,
}

impl Invoice {
    pub fn new(id: InvoiceId, amount: Sats, bolt11: Bolt11, expires_at: UnixTimestamp) -> Self {
        Self {
            id,
            amount,
            bolt11,
            expires_at,
            status: PaymentStatus::Pending,
        }
    }

    pub fn mark_paid(&mut self) {
        self.status = PaymentStatus::Paid;
    }

    pub fn mark_expired(&mut self) {
        self.status = PaymentStatus::Expired;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payment {
    pub invoice_id: InvoiceId,
    pub amount: Sats,
    pub status: PaymentStatus,
    pub received_at: UnixTimestamp,
}

impl Payment {
    pub fn received(invoice_id: InvoiceId, amount: Sats, received_at: UnixTimestamp) -> Self {
        Self {
            invoice_id,
            amount,
            status: PaymentStatus::Paid,
            received_at,
        }
    }
}

fn validate_identifier(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<String, ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::Empty { field });
    }

    if value.len() > max_len {
        return Err(ValidationError::TooLong {
            field,
            max: max_len,
        });
    }

    let valid = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'));

    if !valid {
        return Err(ValidationError::InvalidCharacters { field });
    }

    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sats_rejects_zero_amounts() {
        let result = Sats::new(0);
        assert!(matches!(
            result,
            Err(ValidationError::ZeroAmount {
                field: "amount_sats"
            })
        ));
    }

    #[test]
    fn invoice_id_rejects_whitespace() {
        let result = InvoiceId::new("invoice 1");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidCharacters {
                field: "invoice_id"
            })
        ));
    }

    #[test]
    fn bolt11_must_start_with_ln() {
        let result = Bolt11::new("bitcoin:bc1...");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidBolt11 { field: "bolt11" })
        ));
    }
}
