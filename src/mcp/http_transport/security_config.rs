use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;

const TRUSTED_PROXIES_ENV: &str = "ENGRAM_HTTP_TRUSTED_PROXIES";
pub(super) const MAX_BODY_BYTES_ENV: &str = "ENGRAM_HTTP_MAX_BODY_BYTES";
pub(super) const REQUEST_TIMEOUT_MS_ENV: &str = "ENGRAM_HTTP_REQUEST_TIMEOUT_MS";
pub(super) const DEFAULT_MAX_BODY_BYTES: usize = 1_048_576;
pub(super) const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30_000;
const MAX_FORWARDED_CHAIN: usize = 32;

#[derive(Clone, Debug)]
pub(super) struct HttpSecurityConfig {
    trusted_proxies: Vec<IpCidr>,
    pub(super) max_body_bytes: usize,
    pub(super) request_timeout: std::time::Duration,
}

impl Default for HttpSecurityConfig {
    fn default() -> Self {
        Self {
            trusted_proxies: Vec::new(),
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
            request_timeout: std::time::Duration::from_millis(DEFAULT_REQUEST_TIMEOUT_MS),
        }
    }
}

impl HttpSecurityConfig {
    /// Load HTTP resource limits from the environment.
    ///
    /// Missing variables keep documented safe defaults. Present but invalid,
    /// zero, or over-maximum values fail closed (same policy as WebSocket
    /// resource configuration).
    pub(super) fn from_env() -> Result<Self, String> {
        let trusted_proxies = std::env::var(TRUSTED_PROXIES_ENV)
            .ok()
            .map(|value| Self::parse_trusted_proxies(&value))
            .unwrap_or_default();
        let max_body_bytes = parse_positive_env(
            MAX_BODY_BYTES_ENV,
            DEFAULT_MAX_BODY_BYTES,
            0,
            DEFAULT_MAX_BODY_BYTES.saturating_mul(64),
        )?;
        let request_timeout_ms = parse_positive_env(
            REQUEST_TIMEOUT_MS_ENV,
            DEFAULT_REQUEST_TIMEOUT_MS,
            0,
            3_600_000,
        )?;
        Ok(Self {
            trusted_proxies,
            max_body_bytes,
            request_timeout: std::time::Duration::from_millis(request_timeout_ms),
        })
    }

    fn parse_trusted_proxies(value: &str) -> Vec<IpCidr> {
        value
            .split(',')
            .filter_map(|entry| match IpCidr::parse(entry.trim()) {
                Some(cidr) => Some(cidr),
                None => {
                    if !entry.trim().is_empty() {
                        tracing::warn!(value = %entry.trim(), "ignoring invalid trusted proxy CIDR");
                    }
                    None
                }
            })
            .collect()
    }

    pub(super) fn client_ip(
        &self,
        peer: Option<SocketAddr>,
        headers: &HeaderMap,
    ) -> Option<IpAddr> {
        let peer_ip = peer.map(|address| address.ip())?;
        if !self
            .trusted_proxies
            .iter()
            .any(|cidr| cidr.contains(peer_ip))
        {
            return Some(peer_ip);
        }

        let forwarded = match headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            Some(value) => value,
            None => return Some(peer_ip),
        };
        let chain: Vec<IpAddr> = forwarded
            .split(',')
            .map(str::trim)
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()
            .filter(|chain: &Vec<IpAddr>| !chain.is_empty() && chain.len() <= MAX_FORWARDED_CHAIN)
            .unwrap_or_default();
        if chain.is_empty() {
            return Some(peer_ip);
        }

        // Walk right-to-left across proxy hops. The first address outside the
        // allowlist is the client. All-trusted chains normalize to the leftmost hop.
        chain
            .iter()
            .rev()
            .copied()
            .find(|ip| !self.trusted_proxies.iter().any(|cidr| cidr.contains(*ip)))
            .or_else(|| chain.first().copied())
            .or(Some(peer_ip))
    }
}

fn parse_positive_env<T>(name: &str, default: T, zero: T, maximum: T) -> Result<T, String>
where
    T: Copy + std::fmt::Display + std::str::FromStr + PartialEq + PartialOrd + Default,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let value = match std::env::var(name) {
        Ok(value) => value,
        Err(_) => return Ok(default),
    };
    parse_positive_value(name, &value, zero, maximum)
}

fn parse_positive_value<T>(name: &str, value: &str, zero: T, maximum: T) -> Result<T, String>
where
    T: Copy + std::fmt::Display + std::str::FromStr + PartialEq + PartialOrd + Default,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let parsed = value
        .parse::<T>()
        .map_err(|error| format!("{name} must be a positive integer: {error}"))?;
    if parsed == T::default() || parsed <= zero {
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

    #[test]
    fn defaults_are_bounded() {
        let config = HttpSecurityConfig::default();
        assert_eq!(config.max_body_bytes, DEFAULT_MAX_BODY_BYTES);
        assert_eq!(
            config.request_timeout,
            std::time::Duration::from_millis(DEFAULT_REQUEST_TIMEOUT_MS)
        );
    }

    #[test]
    fn zero_and_invalid_values_fail_closed() {
        assert!(parse_positive_value("TEST", "0", 0_usize, 64).is_err());
        assert!(parse_positive_value("TEST", "unbounded", 0_u64, 100).is_err());
        assert!(parse_positive_value("TEST", "65", 0_usize, 64).is_err());
        assert_eq!(parse_positive_value("TEST", "32", 0_usize, 64).unwrap(), 32);
    }
}

#[derive(Clone, Copy, Debug)]
struct IpCidr {
    network: IpAddr,
    prefix: u8,
}

impl IpCidr {
    fn parse(value: &str) -> Option<Self> {
        let (ip, prefix): (IpAddr, u8) = match value.split_once('/') {
            Some((ip, prefix)) => (ip.parse().ok()?, prefix.parse().ok()?),
            None => {
                let ip: IpAddr = value.parse().ok()?;
                let prefix = if ip.is_ipv4() { 32 } else { 128 };
                return Some(Self {
                    network: ip,
                    prefix,
                });
            }
        };
        let max = if ip.is_ipv4() { 32 } else { 128 };
        (prefix <= max).then_some(Self {
            network: ip,
            prefix,
        })
    }

    fn contains(self, ip: IpAddr) -> bool {
        match (self.network, ip) {
            (IpAddr::V4(network), IpAddr::V4(ip)) => {
                let mask = u32::MAX.checked_shl((32 - self.prefix).into()).unwrap_or(0);
                u32::from(network) & mask == u32::from(ip) & mask
            }
            (IpAddr::V6(network), IpAddr::V6(ip)) => {
                let mask = u128::MAX
                    .checked_shl((128 - self.prefix).into())
                    .unwrap_or(0);
                u128::from(network) & mask == u128::from(ip) & mask
            }
            _ => false,
        }
    }
}
