pub mod auth;
pub mod config;
pub mod error;
pub mod provider;
pub mod routes;
pub mod storage;

use axum::{
    Router,
    routing::{get, post},
};
pub use config::GatewayConfig;
use provider::MockPaymentProvider;
use storage::{IdempotencyStore, InvoiceStore};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub config: GatewayConfig,
    pub invoices: InvoiceStore,
    pub idempotency: IdempotencyStore,
    pub provider: MockPaymentProvider,
}

impl AppState {
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            config,
            invoices: InvoiceStore::default(),
            idempotency: IdempotencyStore::default(),
            provider: MockPaymentProvider,
        }
    }
}

pub fn build_router(config: GatewayConfig) -> Router {
    let state = AppState::new(config);

    Router::new()
        .route("/healthz", get(routes::health))
        .route("/v1/invoices", post(routes::create_invoice))
        .route("/v1/invoices/{invoice_id}", get(routes::get_invoice))
        .route(
            "/webhooks/mock/invoices/{invoice_id}/paid",
            post(routes::mock_mark_invoice_paid),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
