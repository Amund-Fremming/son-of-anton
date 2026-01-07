# Son of Anton - Development Diary

## January 6, 2026

### ZBT-2 / Zigbee2MQTT Setup

**Problem:** Spent hours trying to connect the ZBT-2 adapter to Zigbee2MQTT. Kept getting `HOST_FATAL_ERROR` and `SRSP - SYS - ping after 6000ms` errors.

**Wrong assumptions:**
- Thought I needed to flash specific firmware
- Tried multiple adapter types (`zstack`, `deconz`) before finding the right one

**Solution:**
The fix was in the Zigbee2MQTT configuration:

```yaml
serial:
  adapter: ember
  baudrate: 460800
mqtt:
  server: mqtt://localhost:1883  # localhost since Mosquitto runs in Docker
```

**Key learnings:**
- ZBT-2 (Nabu Casa) uses `ember` adapter type, not `zstack`
- Baud rate must be `460800` (not the default 115200)
- When running Zigbee2MQTT natively on macOS with Mosquitto in Docker, use `localhost`
