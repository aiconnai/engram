use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;

const TRUSTED_PROXIES_ENV: &str = "ENGRAM_HTTP_TRUSTED_PROXIES";
const MAX_FORWARDED_CHAIN: usize = 32;

#[derive(Clone, Debug, Default)]
pub(super) struct HttpSecurityConfig {
    trusted_proxies: Vec<IpCidr>,
}

impl HttpSecurityConfig {
    pub(super) fn from_env() -> Self {
        let trusted_proxies = std::env::var(TRUSTED_PROXIES_ENV)
            .ok()
            .map(|value| Self::parse_trusted_proxies(&value))
            .unwrap_or_default();
        Self { trusted_proxies }
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
