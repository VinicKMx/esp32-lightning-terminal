# Threat Model

| Asset | Threat | Attack surface | Impact | Mitigation | Residual risk |
| --- | --- | --- | --- | --- | --- |
| Device secret | Physical terminal theft | Flash extraction, debug port | Device impersonation | Flash Encryption, debug restrictions, revocation | Advanced physical attacks remain possible |
| Firmware integrity | Modified firmware | OTA, serial flashing, supply chain | False UI, credential theft | Secure Boot, signed OTA, release checks | Compromised signing key is critical |
| Gateway API | Replay message | Network capture | Duplicate invoice action | Timestamped HMAC, idempotency keys | Clock skew handling must be monitored |
| Payment status | False payment event | Provider webhook endpoint | Goods released without payment | Webhook validation, persisted provider state lookup | Provider compromise remains high impact |
| Invoice contents | Invoice substitution | MITM, compromised gateway | Payment to wrong destination | TLS, QR generated from gateway response, provider audit | A compromised gateway can still lie |
| BTCPay API key | Secret leak | Gateway env, logs, filesystem | Provider account compromise | Secret redaction, least privilege, secret manager | Host compromise can expose runtime secrets |
| Device identity | Brute force | Gateway auth endpoint | Device impersonation | High-entropy secrets, rate limiting, lockout | Operational tuning required |
| OTA state | Rollback firmware | OTA metadata, old image | Reintroduction of vulnerabilities | Anti-rollback, signed version metadata | Hardware support and policy must be verified |
| Supply chain | Dependency compromise | Rust crates, ESP-IDF components | Malicious code inclusion | Lockfiles, audits, pinned toolchains | Transitive risk remains |
