use std::net::{IpAddr, SocketAddr};

use axum::http::header::{HOST, ORIGIN};
use axum::http::uri::Authority;
use axum::http::{HeaderMap, Uri};

const TRUSTED_PROXIES_ENV: &str = "ENGRAM_HTTP_TRUSTED_PROXIES";
pub(super) const MAX_BODY_BYTES_ENV: &str = "ENGRAM_HTTP_MAX_BODY_BYTES";
pub(super) const REQUEST_TIMEOUT_MS_ENV: &str = "ENGRAM_HTTP_REQUEST_TIMEOUT_MS";
pub(super) const DEFAULT_MAX_BODY_BYTES: usize = 1_048_576;
pub(super) const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30_000;
pub(super) const ALLOWED_HOSTS_ENV: &str = "ENGRAM_HTTP_ALLOWED_HOSTS";
const CORS_ORIGINS_ENV: &str = "ENGRAM_CORS_ORIGINS";
const MAX_FORWARDED_CHAIN: usize = 32;
/// Names a browser can only reach by actually targeting this machine.
const LOOPBACK_HOSTS: &[&str] = &["localhost", "127.0.0.1", "[::1]"];

#[derive(Clone, Debug)]
pub(super) struct HttpSecurityConfig {
    trusted_proxies: Vec<IpCidr>,
    /// Extra `Host` names accepted on a keyless server (lowercase, no port).
    allowed_hosts: Vec<String>,
    /// Raw `ENGRAM_CORS_ORIGINS`: `None` keeps the loopback-only default.
    cors_origins: Option<String>,
    pub(super) max_body_bytes: usize,
    pub(super) request_timeout: std::time::Duration,
}

impl Default for HttpSecurityConfig {
    fn default() -> Self {
        Self {
            trusted_proxies: Vec::new(),
            allowed_hosts: Vec::new(),
            cors_origins: None,
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
            allowed_hosts: std::env::var(ALLOWED_HOSTS_ENV)
                .map(|value| Self::parse_allowed_hosts(&value))
                .unwrap_or_default(),
            cors_origins: std::env::var(CORS_ORIGINS_ENV).ok(),
            max_body_bytes,
            request_timeout: std::time::Duration::from_millis(request_timeout_ms),
        })
    }

    pub(super) fn parse_allowed_hosts(value: &str) -> Vec<String> {
        value
            .split(',')
            .map(|entry| entry.trim().to_ascii_lowercase())
            .filter(|entry| !entry.is_empty())
            .collect()
    }

    #[cfg(test)]
    pub(super) fn with_browser_policy(mut self, hosts: &str, cors_origins: Option<&str>) -> Self {
        self.allowed_hosts = Self::parse_allowed_hosts(hosts);
        self.cors_origins = cors_origins.map(str::to_string);
        self
    }

    fn host_name_allowed(&self, host: &str) -> bool {
        let host = host.to_ascii_lowercase();
        LOOPBACK_HOSTS.contains(&host.as_str()) || self.allowed_hosts.contains(&host)
    }

    /// Why a request to a keyless server must be refused, if it must.
    ///
    /// Without an API key the only thing separating this server from any web
    /// page the operator visits is the browser's same-origin policy, which DNS
    /// rebinding defeats: the page's own name is re-pointed at 127.0.0.1. Such
    /// a request still carries the attacker's name in `Host` (and in `Origin`
    /// when one is sent), so both must name this machine. Non-browser clients
    /// that send neither header are unaffected.
    pub(super) fn keyless_request_denial(
        &self,
        headers: &HeaderMap,
        uri: &Uri,
    ) -> Option<&'static str> {
        let mut hosts = headers.get_all(HOST).iter();
        let host = match (hosts.next(), hosts.next()) {
            (Some(_), Some(_)) => return Some("multiple Host headers"),
            (Some(value), None) => match value.to_str().ok().map(str::parse::<Authority>) {
                Some(Ok(authority)) => Some(authority.host().to_string()),
                _ => return Some("malformed Host header"),
            },
            // HTTP/2 carries the authority in the request target instead.
            (None, _) => uri.host().map(str::to_string),
        };
        if host.is_some_and(|host| !self.host_name_allowed(&host)) {
            return Some(
                "Host is not a loopback name; set ENGRAM_HTTP_ALLOWED_HOSTS or an API key",
            );
        }

        let mut origins = headers.get_all(ORIGIN).iter();
        let origin = match (origins.next(), origins.next()) {
            (Some(_), Some(_)) => return Some("multiple Origin headers"),
            (Some(value), None) => match value.to_str() {
                Ok(origin) => origin,
                Err(_) => return Some("malformed Origin header"),
            },
            (None, _) => return None,
        };
        if self.origin_allowed(origin) {
            None
        } else {
            Some("Origin is not allowed; set ENGRAM_CORS_ORIGINS or an API key")
        }
    }

    fn origin_allowed(&self, origin: &str) -> bool {
        match self.cors_origins.as_deref().map(str::trim) {
            Some("*") => true,
            Some(list) => list.split(',').any(|entry| entry.trim() == origin),
            // Exact loopback host, any port: a prefix test would also accept
            // `http://localhost.attacker.example`.
            None => origin.parse::<Uri>().is_ok_and(|uri| {
                matches!(uri.scheme_str(), Some("http" | "https"))
                    && uri
                        .host()
                        .is_some_and(|host| LOOPBACK_HOSTS.contains(&host))
            }),
        }
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
