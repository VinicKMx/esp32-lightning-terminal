use async_trait::async_trait;
use terminal_models::{Bolt11, Invoice, InvoiceId, PaymentStatus, Sats, UnixTimestamp};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PaymentProviderError {
    #[error("provider returned invalid data: {0}")]
    InvalidProviderData(String),

    #[error("provider is temporarily unavailable")]
    Unavailable,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_invoice(
        &self,
        amount: Sats,
        ttl_seconds: u64,
    ) -> Result<Invoice, PaymentProviderError>;

    async fn get_invoice_status(
        &self,
        invoice: &Invoice,
    ) -> Result<PaymentStatus, PaymentProviderError>;
}

#[derive(Debug, Clone, Default)]
pub struct MockPaymentProvider;

#[async_trait]
impl PaymentProvider for MockPaymentProvider {
    async fn create_invoice(
        &self,
        amount: Sats,
        ttl_seconds: u64,
    ) -> Result<Invoice, PaymentProviderError> {
        let id = Uuid::new_v4().to_string();
        let invoice_id = InvoiceId::new(id.clone())
            .map_err(|error| PaymentProviderError::InvalidProviderData(error.to_string()))?;
        let compact_id = id.replace('-', "");
        let bolt11 = Bolt11::new(format!("lnbc{}n1p{}", amount.as_u64(), compact_id))
            .map_err(|error| PaymentProviderError::InvalidProviderData(error.to_string()))?;

        Ok(Invoice::new(
            invoice_id,
            amount,
            bolt11,
            UnixTimestamp::now_plus_seconds(ttl_seconds),
        ))
    }

    async fn get_invoice_status(
        &self,
        invoice: &Invoice,
    ) -> Result<PaymentStatus, PaymentProviderError> {
        Ok(invoice.status)
    }
}
