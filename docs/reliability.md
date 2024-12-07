# Reliability

Reliability requirements:

- reboot during active invoice;
- power loss;
- Wi-Fi loss and reconnect;
- gateway unavailability;
- provider unavailability;
- invoice expiration;
- duplicated provider events;
- duplicated HTTP retries;
- lost responses;
- partial persisted state;
- OTA failure and rollback.

Confirmed payments must be persisted by the gateway. Firmware UI confirmation must be recoverable by querying gateway state after reconnect or reboot.

The terminal state machine preserves active invoice context across network loss. Persistent recovery on the device will build on this domain behavior.
