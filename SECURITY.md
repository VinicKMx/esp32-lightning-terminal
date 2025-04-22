# Security Policy

ESP32 Lightning Terminal handles payment flows, so the security surface is the
point of the project rather than a side concern. It is also at an early
foundation stage and makes no production security claims.

The design documents are the authoritative description:

- [Threat model](docs/threat-model.md)
- [Security](docs/security.md)
- [Protocol](docs/protocol.md)
- [OTA](docs/ota.md)

## Current Limits

- Invoice creation and payment confirmation are mocked. No real Lightning
  backend is wired in yet.
- OTA is documented but not signed. Do not deploy the firmware to a device that
  accepts remote updates.
- Device-to-gateway authentication is HMAC-SHA256 over a shared secret. A
  compromised device secret compromises that device's session.
- The project does not implement cryptographic primitives. It composes vetted
  implementations.

## Reporting

Report security-sensitive issues privately before public disclosure. Until a
dedicated security contact exists, open a minimal issue asking for a private
contact path, without publishing exploit details.

Include the affected component, reproduction steps, the firmware and gateway
versions, and the impact.

## Out of Scope

Do not report findings that depend on physical possession of an unprovisioned
device, on secrets committed by the operator, or on a gateway deployed without
transport security. Those are deployment errors, and the documentation says so.
