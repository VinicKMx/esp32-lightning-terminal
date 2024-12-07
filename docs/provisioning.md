# Provisioning

Provisioning configures Wi-Fi, gateway URL, device identity and device credentials.

Initial supported paths should include USB/serial provisioning for development. Production-capable paths may include SoftAP captive portal, BLE provisioning and QR provisioning.

Provisioned values:

- Wi-Fi SSID and credential;
- gateway HTTPS URL;
- `device_id`;
- device secret or private key;
- device profile and allowed capabilities.

Credentials must be stored using ESP32 secure storage features where available. Production provisioning must support revocation and credential rotation.
