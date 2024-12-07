# Testing

## Unit Tests

Host-side unit tests cover:

- domain validation;
- protocol serialization;
- terminal FSM transitions;
- duplicate payment events;
- network loss while waiting for payment;
- invoice expiration.

## Integration Tests

Gateway integration tests should cover:

- device authentication;
- idempotent invoice creation;
- invoice lookup;
- mock payment lifecycle;
- persistence once SQLite is active;
- BTCPay provider adapter behavior.

## Hardware-in-the-loop

Future HIL tests should:

- flash ESP32 firmware;
- wait for serial boot logs;
- configure Wi-Fi;
- create a mock invoice;
- simulate payment;
- verify terminal confirmation.
