use axum::{
    Json,
    body::Bytes,
    extract::{OriginalUri, Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use terminal_models::InvoiceId;
use terminal_protocol::{
    CreateInvoiceRequest, CreateInvoiceResponse, GetInvoiceResponse, HEADER_IDEMPOTENCY_KEY,
    IdempotencyKey,
};
use tracing::{info, warn};

use crate::{AppState, auth::verify_request, error::ApiError, provider::PaymentProvider};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn create_invoice(
    State(state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let device_id = verify_request(&state.config, &headers, &Method::POST, uri.path(), &body)?;
    let idempotency_key = idempotency_key(&headers)?;
    let request: CreateInvoiceRequest = serde_json::from_slice(&body)
        .map_err(|error| ApiError::BadRequest(format!("invalid JSON body: {error}")))?;

    if let Some(invoice_id) = state.idempotency.get(&device_id, &idempotency_key).await {
        if let Some(invoice) = state.invoices.get(&invoice_id).await {
            info!(
                device_id = %device_id,
                invoice_id = %invoice.id,
                "returning idempotent invoice response"
            );
            return Ok((StatusCode::OK, Json(CreateInvoiceResponse::from(&invoice))));
        }

        warn!(
            device_id = %device_id,
            invoice_id = %invoice_id,
            "idempotency record pointed to a missing invoice"
        );
    }

    let invoice = state
        .provider
        .create_invoice(request.amount_sats, state.config.invoice_ttl_seconds)
        .await?;

    state.invoices.insert(invoice.clone()).await;
    state
        .idempotency
        .insert(device_id.clone(), idempotency_key, invoice.id.clone())
        .await;

    info!(
        device_id = %device_id,
        invoice_id = %invoice.id,
        amount_sats = invoice.amount.as_u64(),
        "created mock Lightning invoice"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateInvoiceResponse::from(&invoice)),
    ))
}

pub async fn get_invoice(
    State(state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
    Path(invoice_id): Path<String>,
) -> Result<Json<GetInvoiceResponse>, ApiError> {
    verify_request(&state.config, &headers, &Method::GET, uri.path(), &[])?;
    let invoice_id = InvoiceId::new(invoice_id)
        .map_err(|error| ApiError::BadRequest(format!("invalid invoice id: {error}")))?;

    let invoice = state
        .invoices
        .get(&invoice_id)
        .await
        .ok_or_else(|| ApiError::NotFound("invoice not found".to_owned()))?;

    Ok(Json(GetInvoiceResponse::from(&invoice)))
}

pub async fn mock_mark_invoice_paid(
    State(state): State<AppState>,
    Path(invoice_id): Path<String>,
) -> Result<Json<GetInvoiceResponse>, ApiError> {
    let invoice_id = InvoiceId::new(invoice_id)
        .map_err(|error| ApiError::BadRequest(format!("invalid invoice id: {error}")))?;
    let invoice = state
        .invoices
        .mark_paid(&invoice_id)
        .await
        .ok_or_else(|| ApiError::NotFound("invoice not found".to_owned()))?;

    info!(
        invoice_id = %invoice.id,
        amount_sats = invoice.amount.as_u64(),
        "mock invoice marked paid"
    );

    Ok(Json(GetInvoiceResponse::from(&invoice)))
}

fn idempotency_key(headers: &HeaderMap) -> Result<IdempotencyKey, ApiError> {
    let value = headers
        .get(HEADER_IDEMPOTENCY_KEY)
        .ok_or_else(|| ApiError::BadRequest("missing idempotency-key header".to_owned()))?
        .to_str()
        .map_err(|_| ApiError::BadRequest("invalid idempotency-key header".to_owned()))?;

    IdempotencyKey::new(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid idempotency key: {error}")))
}
