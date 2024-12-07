# Security

This project is designed as a connected embedded payment system. The terminal must not store administrative Lightning provider credentials.

## Current Controls

- Device requests use HMAC-SHA256 authentication.
- The gateway validates device id, timestamp skew and request signature.
- Invoice creation requires idempotency keys.
- Shared models validate domain identifiers, BOLT11 shape and sats bounds.
- Logs must not include secrets, API keys, private keys or raw credentials.

## Production Requirements

- TLS with certificate validation.
- Unique device identity and revocation.
- Secure credential storage on ESP32.
- Secure Boot.
- Flash Encryption.
- Signed OTA images.
- OTA rollback.
- Downgrade protection when supported.
- Debug interface restrictions for production builds.
- Gateway rate limiting.
- Provider webhook validation.
- Strict input validation on all protocol boundaries.

## Secret Handling

Secrets belong in environment variables, secure stores or provisioning channels. They must not be committed.

The `.env.example` file documents configuration names without real secrets.
