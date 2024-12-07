# Terminal Gateway Protocol

Current version: `v1`.

All terminal requests are sent over HTTPS in production.

## Authentication

Initial device authentication uses HMAC-SHA256.

Required headers:

- `x-terminal-device-id`;
- `x-terminal-timestamp`;
- `x-terminal-signature`;
- `idempotency-key` for invoice creation.

The signature payload is:

```text
METHOD
PATH
UNIX_TIMESTAMP
SHA256_HEX(BODY)
```

The gateway rejects unknown devices, invalid signatures and timestamps outside the configured clock skew.

Future production identity should evolve to per-device certificates and private keys.

## Create Invoice

```http
POST /v1/invoices
```

Request:

```json
{
  "amount_sats": 10000
}
```

Response:

```json
{
  "invoice_id": "01J...",
  "amount_sats": 10000,
  "bolt11": "lnbc...",
  "expires_at": 1786383000
}
```

The request must include an idempotency key. Retrying with the same device and idempotency key returns the original invoice.

## Get Invoice

```http
GET /v1/invoices/{invoice_id}
```

Response:

```json
{
  "invoice_id": "01J...",
  "status": "paid",
  "amount_sats": 10000
}
```

Payment status values:

- `pending`;
- `paid`;
- `expired`;
- `failed`.

## Compatibility

Protocol changes must preserve explicit versioning. Breaking changes require a new versioned path.
