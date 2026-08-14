//! WebSocket Origin allowlist policy.

use std::collections::HashSet;

use axum::http::{header::ORIGIN, HeaderMap, Uri};

pub(super) const WS_ALLOWED_ORIGINS_ENV: &str = "ENGRAM_WS_ALLOWED_ORIGINS";

pub(super) fn origin_is_allowed(headers: &HeaderMap, allowed_origins: &HashSet<String>) -> bool {
    let mut origins = headers.get_all(ORIGIN).iter();
    let Some(origin) = origins.next() else {
        return true;
    };
    if origins.next().is_some() {
        return false;
    }
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    allowed_origins.contains(origin)
}

pub(super) fn parse_origin_allowlist(raw: Option<&str>) -> Result<HashSet<String>, String> {
    let Some(raw) = raw else {
        return Ok(HashSet::new());
    };

    let mut origins = HashSet::new();
    for entry in raw.split(',') {
        let origin = entry.trim();
        if origin.is_empty() || origin == "*" {
            return Err(format!(
                "{WS_ALLOWED_ORIGINS_ENV} must contain explicit origins without wildcards"
            ));
        }
        let uri = origin
            .parse::<Uri>()
            .map_err(|_| format!("{WS_ALLOWED_ORIGINS_ENV} contains an invalid origin"))?;
        if !matches!(uri.scheme_str(), Some("http" | "https"))
            || uri
                .authority()
                .is_none_or(|authority| authority.as_str().contains('@'))
            || uri.query().is_some()
            || !matches!(uri.path(), "" | "/")
        {
            return Err(format!(
                "{WS_ALLOWED_ORIGINS_ENV} origins must be scheme-and-authority values"
            ));
        }
        origins.insert(origin.trim_end_matches('/').to_string());
    }
    Ok(origins)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wildcard_and_accepts_explicit_origins() {
        assert!(parse_origin_allowlist(Some("*")).is_err());
        let set = parse_origin_allowlist(Some("https://app.example")).unwrap();
        assert!(set.contains("https://app.example"));
    }
}
