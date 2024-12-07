use std::{collections::HashMap, sync::Arc};
use terminal_models::{DeviceId, Invoice, InvoiceId};
use terminal_protocol::IdempotencyKey;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct InvoiceStore {
    invoices: Arc<RwLock<HashMap<InvoiceId, Invoice>>>,
}

impl InvoiceStore {
    pub async fn insert(&self, invoice: Invoice) {
        self.invoices
            .write()
            .await
            .insert(invoice.id.clone(), invoice);
    }

    pub async fn get(&self, invoice_id: &InvoiceId) -> Option<Invoice> {
        self.invoices.read().await.get(invoice_id).cloned()
    }

    pub async fn mark_paid(&self, invoice_id: &InvoiceId) -> Option<Invoice> {
        let mut invoices = self.invoices.write().await;
        let invoice = invoices.get_mut(invoice_id)?;
        invoice.mark_paid();
        Some(invoice.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct IdempotencyStore {
    records: Arc<RwLock<HashMap<IdempotencyScope, InvoiceId>>>,
}

impl IdempotencyStore {
    pub async fn insert(
        &self,
        device_id: DeviceId,
        idempotency_key: IdempotencyKey,
        invoice_id: InvoiceId,
    ) {
        self.records.write().await.insert(
            IdempotencyScope {
                device_id,
                idempotency_key,
            },
            invoice_id,
        );
    }

    pub async fn get(
        &self,
        device_id: &DeviceId,
        idempotency_key: &IdempotencyKey,
    ) -> Option<InvoiceId> {
        self.records
            .read()
            .await
            .get(&IdempotencyScope {
                device_id: device_id.clone(),
                idempotency_key: idempotency_key.clone(),
            })
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IdempotencyScope {
    device_id: DeviceId,
    idempotency_key: IdempotencyKey,
}
