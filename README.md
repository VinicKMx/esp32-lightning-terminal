# ESP32 Lightning Terminal

ESP32 Lightning Terminal is an open-source reference project for physical
Bitcoin Lightning payment terminals built with ESP32-S3, Rust and ESP-IDF.

The goal is not just to display a Lightning invoice on a small screen. The goal
is to build a production-minded embedded payment framework that can be reused
for point-of-sale terminals, donation boxes, vending machines, arcade cabinets,
dispensers, coffee machines, relays, locks and other pay-per-use devices.

## Status

Early foundation stage.

What works today:

- Rust monorepo with shared domain crates;
- ESP32-S3 firmware skeleton compiling for `xtensa-esp32s3-espidf`;
- explicit terminal state machine with unit tests;
- versioned terminal/gateway protocol types;
- Axum gateway with HMAC device authentication;
- idempotent mock invoice creation;
- mock invoice payment confirmation endpoint;
- Docker Compose gateway entrypoint;
- initial architecture, security, protocol, OTA and hardware documentation.

What is still planned:

- real display driver integration;
- local QR rendering on device;
- Wi-Fi manager;
- firmware HTTP client;
- persistent gateway storage;
- BTCPay Server provider;
- provisioning;
- recovery logic;
- signed OTA;
- hardware-in-the-loop tests;
- reference PCB.

## Architecture

```text
ESP32-S3 firmware
  Rust + ESP-IDF + FreeRTOS + embedded-graphics
        |
        | HTTPS protocol v1
        v
Terminal Gateway
  Rust + Tokio + Axum + provider abstraction
        |
        | provider API
        v
Lightning provider
  BTCPay Server first, other backends later
```

The firmware talks to the gateway through this project's own protocol. It does
not depend conceptually on BTCPay Server. Provider-specific integrations live in
the gateway so future backends such as LNbits, LND, Core Lightning or phoenixd
can be added without changing the core firmware domain.

## Repository Layout

```text
firmware/                 ESP32-S3 Rust/ESP-IDF firmware
gateway/                  Rust gateway service
crates/terminal-models    Strong domain types
crates/terminal-core      Hardware-independent terminal state machine
crates/terminal-protocol  Versioned terminal/gateway protocol
docs/                     Product architecture and security documentation
hardware/                 Reference hardware documentation placeholders
examples/                 Reusable application profiles
tests/                    Integration and future HIL test suites
deploy/                   Docker assets
```

## Domain Model

The shared domain uses strong types for payment concepts:

- `Sats`;
- `DeviceId`;
- `InvoiceId`;
- `Bolt11`;
- `Invoice`;
- `Payment`;
- `PaymentStatus`.

The state machine is explicit and testable outside the ESP32. Current states
include booting, provisioning, connecting, idle, entering amount, creating
invoice, awaiting payment, payment received, expired, network unavailable,
error and updating.

## Quick Start

Run host-side validation:

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude lightning-terminal-firmware --all-targets -- -D warnings
cargo test --workspace --exclude lightning-terminal-firmware
```

Run the gateway:

```bash
cp .env.example .env
cargo run -p lightning-terminal-gateway
```

Or with Docker:

```bash
docker compose up
```

Health check:

```bash
curl http://127.0.0.1:3000/healthz
```

## Firmware

The firmware targets `xtensa-esp32s3-espidf`.

Build:

```bash
cd firmware
cargo build
```

Flash and monitor an ESP32-S3 board with 8 MiB flash:

```bash
cargo espflash flash --chip esp32s3 --flash-size 8mb --partition-table partitions.csv --monitor
```

Generate a merged image without a connected board:

```bash
cargo espflash save-image --chip esp32s3 --flash-size 8mb --merge --partition-table partitions.csv /tmp/lightning-terminal-firmware-esp32s3.bin
```

The root workspace pins host Rust. The `firmware/` directory uses the `esp`
toolchain and pins ESP-IDF through `esp-idf-sys` metadata.

## Gateway

The gateway currently provides:

- `GET /healthz`;
- `POST /v1/invoices`;
- `GET /v1/invoices/{invoice_id}`;
- HMAC device authentication;
- `idempotency-key` handling;
- in-memory mock payment provider;
- mock endpoint for local payment confirmation.

BTCPay credentials must only be configured on the gateway. The terminal must
never store BTCPay administrative credentials.

## Security

This project treats the terminal as an embedded device connected to financial
infrastructure. Security work is tracked from the start, including:

- per-device identity;
- TLS and certificate validation;
- strict protocol validation;
- idempotent payment operations;
- provider webhook validation;
- secure credential storage;
- Secure Boot;
- Flash Encryption;
- signed OTA and rollback;
- debug restrictions for production builds.

See:

- [Security](docs/security.md);
- [Threat Model](docs/threat-model.md).

## Documentation

- [Architecture](docs/architecture.md)
- [Protocol](docs/protocol.md)
- [Security](docs/security.md)
- [Threat Model](docs/threat-model.md)
- [Provisioning](docs/provisioning.md)
- [OTA](docs/ota.md)
- [Hardware](docs/hardware.md)
- [Testing](docs/testing.md)
- [Reliability](docs/reliability.md)

## Examples

Planned reusable profiles:

- `coffee-pos`;
- `donation-box`;
- `vending-machine`;
- `lightning-switch`.

## Roadmap

1. Monorepo foundation.
2. Firmware boot, display, Wi-Fi, HTTP client and QR rendering.
3. Terminal core recovery logic.
4. Protocol compatibility tests.
5. Gateway persistence and mock lifecycle.
6. BTCPay provider.
7. End-to-end Lightning payment.
8. Reliability hardening.
9. Secure provisioning.
10. OTA and production security.
11. Hardware-in-the-loop tests.
12. Reference PCB.

## Buy Me a Coffee

If this project helped you, you can send a few sats over Lightning:

`maquinalab@walletofsatoshi.com`

<img src="assets/lightning-donation-qr.svg" alt="Lightning donation QR code" width="180">

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option.
