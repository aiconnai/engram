//! Bounded WebSocket resource configuration.

use std::time::Duration;

/// Maximum simultaneous upgraded WebSocket connections.
pub(super) const MAX_CONNECTIONS_ENV: &str = "ENGRAM_WS_MAX_CONNECTIONS";
/// Maximum bytes accepted in one inbound WebSocket message or frame.
pub(super) const MAX_MESSAGE_BYTES_ENV: &str = "ENGRAM_WS_MAX_MESSAGE_BYTES";
/// Maximum time an upgraded client may remain silent between inbound frames.
pub(super) const READ_IDLE_TIMEOUT_ENV: &str = "ENGRAM_WS_READ_IDLE_TIMEOUT_SECONDS";

const DEFAULT_MAX_CONNECTIONS: usize = 128;
const DEFAULT_MAX_MESSAGE_BYTES: usize = 64 * 1024;
const DEFAULT_READ_IDLE_TIMEOUT_SECONDS: u64 = 60;
const MAX_CONFIGURED_CONNECTIONS: usize = 10_000;
const MAX_CONFIGURED_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_CONFIGURED_IDLE_SECONDS: u64 = 3_600;

/// Fail-closed limits used by the realtime transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RealtimeConfig {
    pub(super) max_connections: usize,
    pub(super) max_message_bytes: usize,
    pub(super) read_idle_timeout: Duration,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            read_idle_timeout: Duration::from_secs(DEFAULT_READ_IDLE_TIMEOUT_SECONDS),
        }
    }
}

impl RealtimeConfig {
    pub(super) fn from_env() -> Result<Self, String> {
        Self::from_values(|name| std::env::var(name).ok())
    }

    fn from_values(mut get: impl FnMut(&str) -> Option<String>) -> Result<Self, String> {
        Ok(Self {
            max_connections: parse_nonzero(
                MAX_CONNECTIONS_ENV,
                get(MAX_CONNECTIONS_ENV),
                DEFAULT_MAX_CONNECTIONS,
                MAX_CONFIGURED_CONNECTIONS,
            )?,
            max_message_bytes: parse_nonzero(
                MAX_MESSAGE_BYTES_ENV,
                get(MAX_MESSAGE_BYTES_ENV),
                DEFAULT_MAX_MESSAGE_BYTES,
                MAX_CONFIGURED_MESSAGE_BYTES,
            )?,
            read_idle_timeout: Duration::from_secs(parse_nonzero(
                READ_IDLE_TIMEOUT_ENV,
                get(READ_IDLE_TIMEOUT_ENV),
                DEFAULT_READ_IDLE_TIMEOUT_SECONDS,
                MAX_CONFIGURED_IDLE_SECONDS,
            )?),
        })
    }
}

fn parse_nonzero<T>(name: &str, raw: Option<String>, default: T, maximum: T) -> Result<T, String>
where
    T: std::fmt::Display + std::str::FromStr + PartialEq + PartialOrd + Default,
{
    let Some(raw) = raw else {
        return Ok(default);
    };
    let parsed = raw
        .parse::<T>()
        .map_err(|_| format!("{name} must be a positive integer"))?;
    if parsed == T::default() {
        return Err(format!("{name} must be greater than zero"));
    }
    if parsed > maximum {
        return Err(format!("{name} must not exceed {maximum}"));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn defaults_are_safe_and_bounded() {
        let config = RealtimeConfig::default();
        assert_eq!(config.max_connections, 128);
        assert_eq!(config.max_message_bytes, 64 * 1024);
        assert_eq!(config.read_idle_timeout, Duration::from_secs(60));
    }

    #[test]
    fn overrides_parse_and_zero_or_invalid_values_fail_closed() {
        let values = HashMap::from([
            (MAX_CONNECTIONS_ENV, "4".to_string()),
            (MAX_MESSAGE_BYTES_ENV, "1024".to_string()),
            (READ_IDLE_TIMEOUT_ENV, "5".to_string()),
        ]);
        let config = RealtimeConfig::from_values(|name| values.get(name).cloned()).unwrap();
        assert_eq!(config.max_connections, 4);
        assert_eq!(config.max_message_bytes, 1024);
        assert_eq!(config.read_idle_timeout, Duration::from_secs(5));

        for (name, value) in [
            (MAX_CONNECTIONS_ENV, "0"),
            (MAX_MESSAGE_BYTES_ENV, "not-a-number"),
            (READ_IDLE_TIMEOUT_ENV, "0"),
            (MAX_CONNECTIONS_ENV, "10001"),
            (MAX_MESSAGE_BYTES_ENV, "16777217"),
            (READ_IDLE_TIMEOUT_ENV, "3601"),
        ] {
            assert!(RealtimeConfig::from_values(|candidate| {
                (candidate == name).then(|| value.to_string())
            })
            .is_err());
        }
    }
}
