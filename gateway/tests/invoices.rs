use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use lightning_terminal_gateway::{GatewayConfig, auth::sign_request, build_router};
use terminal_models::{DeviceId, PaymentStatus, UnixTimestamp};
use terminal_protocol::{
    CreateInvoiceResponse, GetInvoiceResponse, HEADER_DEVICE_ID, HEADER_IDEMPOTENCY_KEY,
    HEADER_SIGNATURE, HEADER_TIMESTAMP,
};
use tower::ServiceExt;

fn test_config() -> Result<GatewayConfig, Box<dyn std::error::Error>> {
    Ok(GatewayConfig {
        bind_addr: "127.0.0.1:3000".parse()?,
        device_id: DeviceId::new("dev-terminal-001")?,
        device_secret: "test-secret".to_owned(),
        invoice_ttl_seconds: 900,
        auth_max_skew_seconds: 300,
    })
}

fn signed_request(
    config: &GatewayConfig,
    method: Method,
    path: &str,
    body: &str,
    idempotency_key: Option<&str>,
) -> Result<Request<Body>, Box<dyn std::error::Error>> {
    let timestamp = UnixTimestamp::now();
    let body = body.as_bytes().to_vec();
    let signature = sign_request(&config.device_secret, &method, path, timestamp, &body)?;
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(HEADER_DEVICE_ID, config.device_id.as_str())
        .header(HEADER_TIMESTAMP, timestamp.to_string())
        .header(HEADER_SIGNATURE, signature);

    if let Some(key) = idempotency_key {
        builder = builder.header(HEADER_IDEMPOTENCY_KEY, key);
    }

    Ok(builder.body(Body::from(body))?)
}

async fn decode_response<T>(
    response: axum::response::Response,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = to_bytes(response.into_body(), usize::MAX).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[tokio::test]
async fn invoice_lifecycle_is_authenticated_and_idempotent()
-> Result<(), Box<dyn std::error::Error>> {
    let config = test_config()?;
    let app: Router = build_router(config.clone());
    let request_body = r#"{"amount_sats":10000}"#;

    let create_response = app
        .clone()
        .oneshot(signed_request(
            &config,
            Method::POST,
            "/v1/invoices",
            request_body,
            Some("retry-key-1"),
        )?)
        .await?;
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created: CreateInvoiceResponse = decode_response(create_response).await?;

    let retry_response = app
        .clone()
        .oneshot(signed_request(
            &config,
            Method::POST,
            "/v1/invoices",
            request_body,
            Some("retry-key-1"),
        )?)
        .await?;
    assert_eq!(retry_response.status(), StatusCode::OK);
    let retried: CreateInvoiceResponse = decode_response(retry_response).await?;
    assert_eq!(created.invoice_id, retried.invoice_id);

    let invoice_path = format!("/v1/invoices/{}", created.invoice_id);
    let get_response = app
        .clone()
        .oneshot(signed_request(
            &config,
            Method::GET,
            &invoice_path,
            "",
            None,
        )?)
        .await?;
    assert_eq!(get_response.status(), StatusCode::OK);
    let invoice: GetInvoiceResponse = decode_response(get_response).await?;
    assert_eq!(invoice.status, PaymentStatus::Pending);

    let paid_path = format!("/webhooks/mock/invoices/{}/paid", created.invoice_id);
    let paid_response = app
        .oneshot(Request::post(paid_path).body(Body::empty())?)
        .await?;
    assert_eq!(paid_response.status(), StatusCode::OK);
    let paid: GetInvoiceResponse = decode_response(paid_response).await?;
    assert_eq!(paid.status, PaymentStatus::Paid);

    Ok(())
}
