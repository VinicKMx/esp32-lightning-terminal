# Architecture

The system is split into firmware, shared domain crates and a gateway.

```text
ESP32-S3 firmware
  UI, Wi-Fi, local QR rendering, storage, OTA
        |
        | HTTPS protocol v1
        v
Terminal Gateway
  authentication, invoice orchestration, persistence, provider adapters
        |
        v
Lightning provider
  BTCPay Server first, LNbits/LND/Core Lightning/phoenixd later
```

## Boundaries

Domain code owns payment concepts and terminal state transitions. It does not depend on ESP-IDF, GPIO, Wi-Fi, display drivers, BTCPay, HTTP or a database.

Application code orchestrates state transitions and maps infrastructure events into domain events.

Infrastructure code owns HTTP, storage, payment providers, retries, webhooks and observability.

Platform code owns ESP-IDF, FreeRTOS, display drivers, Wi-Fi, NVS, OTA and hardware-specific details.

UI code maps terminal states to screens. The initial production direction is `embedded-graphics` because it is Rust-native and works well for small embedded displays.

## Current Crates

- `terminal-models`: strong domain types and invoice/payment models.
- `terminal-core`: explicit terminal FSM.
- `terminal-protocol`: versioned terminal/gateway DTOs and protocol headers.
- `lightning-terminal-gateway`: Axum gateway with mock provider.
- `lightning-terminal-firmware`: ESP32-S3 firmware skeleton.
