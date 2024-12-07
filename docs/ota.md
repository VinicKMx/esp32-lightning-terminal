# OTA

The firmware partition table reserves factory, `ota_0` and `ota_1` application slots plus `otadata`.

OTA requirements:

- versioned firmware metadata;
- signed images;
- image validation before boot;
- health validation after boot;
- rollback to last known good image;
- downgrade protection where supported;
- logs for OTA state transitions without leaking secrets.

The initial firmware skeleton enables rollback support in `sdkconfig.defaults`. Secure Boot and Flash Encryption are documented production requirements and must be enabled during secure hardware bring-up.
