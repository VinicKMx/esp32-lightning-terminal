CREATE TABLE devices (
    device_id TEXT PRIMARY KEY,
    secret_hash TEXT NOT NULL,
    revoked_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE invoices (
    invoice_id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL REFERENCES devices(device_id),
    amount_sats INTEGER NOT NULL CHECK (amount_sats > 0),
    bolt11 TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'paid', 'expired', 'failed')),
    provider TEXT NOT NULL,
    provider_invoice_id TEXT,
    idempotency_key TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (device_id, idempotency_key)
);

CREATE TABLE invoice_events (
    event_id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL REFERENCES invoices(invoice_id),
    source TEXT NOT NULL,
    status TEXT NOT NULL,
    received_at INTEGER NOT NULL,
    payload_json TEXT NOT NULL
);
