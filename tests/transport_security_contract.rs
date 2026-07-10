//! Contract tests for the transport security release matrix.
//!
//! These tests intentionally validate a static fixture rather than production
//! transport behavior. Later transport tests can cite fixture row IDs as the
//! release-approved boundary matrix.
//!
//! allow: SIZE_OK — contract-test schema and negative probes are intentionally
//! co-located in this owned file so rollback removes only the fixture/test pair.

use std::collections::BTreeSet;

use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/transport_security.json");
const CONTRACT_VERSION: &str = "transport-security-contract-v1";
const PUBLIC_NO_KEY_ROW_ID: &str = "TSC-HTTP-PUBLIC-NO-KEY-REFUSED";

fn parse_contract(raw: &str) -> Result<Value, String> {
    let value = serde_json::from_str(raw).map_err(|err| format!("invalid JSON: {err}"))?;
    validate_contract(value)
}

fn parse_contract_value(value: Value) -> Result<Value, String> {
    validate_contract(value)
}

fn validate_contract(value: Value) -> Result<Value, String> {
    let version = required_str(&value, &["version"])?;
    if version != CONTRACT_VERSION {
        return Err(format!("unsupported contract version: {version}"));
    }

    let required_ids = required_string_set(&value, &["required_row_ids"])?;
    let rows = required_array(&value, &["rows"])?;
    let mut row_ids = BTreeSet::new();
    let mut coverage = BTreeSet::new();

    for row in rows {
        let row_id = validate_row(row)?;
        if !row_ids.insert(row_id) {
            return Err(format!("duplicate row id: {row_id}"));
        }
        record_coverage(row, &mut coverage)?;
    }

    require_exact_rows(&required_ids, &row_ids)?;
    validate_coverage(&coverage)?;
    Ok(value)
}

fn validate_row(row: &Value) -> Result<&str, String> {
    for field in ["transport", "auth", "bind", "failure"] {
        require_present(row, &[field])?;
    }

    let row_id = required_str(row, &["id"])?;
    for path in [
        &["surface"][..],
        &["threat_model_row"],
        &["bind", "address"],
        &["auth", "server_key"],
        &["auth", "client_credential"],
        &["auth", "scope"],
        &["auth", "proxy_trust"],
        &["expected", "startup"],
        &["expected", "request"],
        &["expected", "status"],
        &["failure", "kind"],
        &["failure", "signal"],
        &["failure", "must_not"],
        &["rollback"],
    ] {
        required_str(row, path)?;
    }

    if !row_id.starts_with("TSC-") {
        return Err(format!("row id is not citeable: {row_id}"));
    }
    if !required_str(row, &["threat_model_row"])?.starts_with("TM-") {
        return Err(format!("{row_id} has invalid threat model row"));
    }
    validate_references(row, row_id)?;
    validate_transport(row, row_id)?;
    validate_bind(row, row_id)?;
    Ok(row_id)
}

fn validate_references(row: &Value, row_id: &str) -> Result<(), String> {
    let references = required_array(row, &["references"])?;
    if references.is_empty() {
        return Err(format!("{row_id} must carry references"));
    }
    for reference in references {
        if reference
            .as_str()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(format!("{row_id} must carry non-empty references"));
        }
    }
    Ok(())
}

fn validate_transport(row: &Value, row_id: &str) -> Result<(), String> {
    let transport = required_str(row, &["transport"])?;
    if !["stdio", "http_mcp", "sse_events", "websocket", "grpc"].contains(&transport) {
        return Err(format!(
            "unknown variant `{transport}` for transport in {row_id}"
        ));
    }
    Ok(())
}

fn validate_bind(row: &Value, row_id: &str) -> Result<(), String> {
    let mode = required_str(row, &["bind", "mode"])?;
    let public = row
        .pointer("/bind/public")
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing field `public` in {row_id}.bind"))?;
    match (mode, public) {
        ("public", true) | ("local_process" | "loopback" | "disabled", false) => Ok(()),
        ("public", false) => Err(format!("{row_id} marks public bind as non-public")),
        ("local_process" | "loopback" | "disabled", true) => {
            Err(format!("{row_id} marks a non-public bind as public"))
        }
        (other, _) => Err(format!("unknown bind mode `{other}` in {row_id}")),
    }
}

fn required_str<'a>(value: &'a Value, path: &[&str]) -> Result<&'a str, String> {
    let found = path_value(value, path)?;
    let text = found
        .as_str()
        .ok_or_else(|| format!("field `{}` must be a string", path.join(".")))?;
    if text.trim().is_empty() {
        return Err(format!("field `{}` must not be empty", path.join(".")));
    }
    Ok(text)
}

fn required_array<'a>(value: &'a Value, path: &[&str]) -> Result<&'a Vec<Value>, String> {
    path_value(value, path)?
        .as_array()
        .ok_or_else(|| format!("field `{}` must be an array", path.join(".")))
}

fn required_string_set<'a>(value: &'a Value, path: &[&str]) -> Result<BTreeSet<&'a str>, String> {
    let mut set = BTreeSet::new();
    for item in required_array(value, path)? {
        let text = item
            .as_str()
            .ok_or_else(|| format!("field `{}` must contain strings", path.join(".")))?;
        if text.trim().is_empty() {
            return Err(format!(
                "field `{}` contains an empty string",
                path.join(".")
            ));
        }
        if !set.insert(text) {
            return Err(format!(
                "field `{}` contains duplicate id: {text}",
                path.join(".")
            ));
        }
    }
    Ok(set)
}

fn require_present(value: &Value, path: &[&str]) -> Result<(), String> {
    path_value(value, path).map(|_| ())
}

fn path_value<'a>(value: &'a Value, path: &[&str]) -> Result<&'a Value, String> {
    let pointer = format!("/{}", path.join("/"));
    value
        .pointer(&pointer)
        .ok_or_else(|| format!("missing field `{}`", path.join(".")))
}

fn require_exact_rows<'a>(
    required_ids: &BTreeSet<&'a str>,
    row_ids: &BTreeSet<&'a str>,
) -> Result<(), String> {
    if row_ids == required_ids {
        return Ok(());
    }
    let missing = required_ids
        .difference(row_ids)
        .copied()
        .collect::<Vec<_>>();
    let unexpected = row_ids
        .difference(required_ids)
        .copied()
        .collect::<Vec<_>>();
    Err(format!(
        "row coverage mismatch; missing required rows: {missing:?}; unexpected rows: {unexpected:?}"
    ))
}

fn record_coverage<'a>(
    row: &'a Value,
    coverage: &mut BTreeSet<(&'static str, &'a str)>,
) -> Result<(), String> {
    for (label, path) in [
        ("transport", &["transport"][..]),
        ("bind mode", &["bind", "mode"]),
        ("server key", &["auth", "server_key"]),
        ("client credential", &["auth", "client_credential"]),
        ("proxy trust", &["auth", "proxy_trust"]),
        ("startup result", &["expected", "startup"]),
        ("request result", &["expected", "request"]),
        ("failure kind", &["failure", "kind"]),
    ] {
        coverage.insert((label, required_str(row, path)?));
    }
    Ok(())
}

fn validate_coverage(coverage: &BTreeSet<(&str, &str)>) -> Result<(), String> {
    for (label, values) in [
        (
            "transport",
            &["stdio", "http_mcp", "sse_events", "websocket", "grpc"][..],
        ),
        (
            "bind mode",
            &["local_process", "loopback", "public", "disabled"],
        ),
        ("server key", &["not_applicable", "absent", "configured"]),
        (
            "client credential",
            &[
                "not_applicable",
                "missing",
                "invalid",
                "valid",
                "malformed_scope",
                "overbroad_scope",
                "spoofed_proxy_header",
            ],
        ),
        (
            "proxy trust",
            &[
                "not_applicable",
                "untrusted_direct",
                "trusted_proxy",
                "missing_trusted_proxy",
            ],
        ),
        ("startup result", &["allowed", "refused", "disabled"]),
        (
            "request result",
            &[
                "not_applicable",
                "success",
                "unauthorized",
                "unauthenticated",
                "service_unavailable",
                "proxy_blocked",
                "proxy_accepted",
                "proxy_ignored",
                "fail_closed",
            ],
        ),
        (
            "failure kind",
            &[
                "none",
                "startup_refused",
                "unauthorized",
                "unauthenticated",
                "service_unavailable",
                "proxy_required",
                "disabled",
                "proxy_spoof_ignored",
                "fail_closed",
            ],
        ),
    ] {
        for value in values {
            if !coverage.contains(&(label, *value)) {
                return Err(format!("missing {label} coverage: {value}"));
            }
        }
    }
    Ok(())
}

#[test]
fn approved_fixture_parses_and_covers_transport_matrix() {
    // Given: the approved transport security matrix fixture.
    // When: the fixture is parsed and validated through the contract schema.
    let contract = parse_contract(FIXTURE).expect("transport security fixture should validate");

    // Then: every release row is present and citeable by later transport tests.
    assert_eq!(
        required_array(&contract, &["rows"]).unwrap().len(),
        required_array(&contract, &["required_row_ids"])
            .unwrap()
            .len()
    );
    assert!(
        required_array(&contract, &["rows"])
            .unwrap()
            .iter()
            .any(|row| row["id"].as_str() == Some(PUBLIC_NO_KEY_ROW_ID)),
        "fixture must include the non-loopback/no-key refusal row"
    );
}

#[test]
fn missing_required_row_fields_are_rejected() {
    for field in ["transport", "auth", "bind", "failure"] {
        // Given: a temp fixture with one required row field removed.
        let mut fixture: Value = serde_json::from_str(FIXTURE).expect("fixture JSON");
        fixture["rows"][0]
            .as_object_mut()
            .expect("first row object")
            .remove(field);

        // When: the temp fixture is parsed.
        let err = parse_contract_value(fixture).expect_err("missing required field should fail");

        // Then: validation rejects the missing contract field.
        assert!(
            err.contains(&format!("missing field `{field}`")),
            "unexpected error for missing {field}: {err}"
        );
    }
}

#[test]
fn malformed_matrix_values_are_rejected() {
    // Given: a temp fixture with a malformed transport enum value.
    let mut fixture: Value = serde_json::from_str(FIXTURE).expect("fixture JSON");
    fixture["rows"][0]["transport"] = Value::String("carrier_pigeon".to_string());

    // When: the temp fixture is parsed.
    let err = parse_contract_value(fixture).expect_err("malformed transport should fail");

    // Then: the parser rejects the malformed matrix value.
    assert!(
        err.contains("unknown variant `carrier_pigeon`"),
        "unexpected malformed value error: {err}"
    );
}

#[test]
fn deleting_non_loopback_no_key_row_fails_coverage() {
    // Given: a temp fixture with the public HTTP/no-key refusal row deleted.
    let mut fixture: Value = serde_json::from_str(FIXTURE).expect("fixture JSON");
    let rows = fixture["rows"].as_array_mut().expect("rows array");
    rows.retain(|row| row["id"].as_str() != Some(PUBLIC_NO_KEY_ROW_ID));

    // When: the temp fixture is parsed.
    let err = parse_contract_value(fixture).expect_err("missing coverage row should fail");

    // Then: coverage validation identifies the deleted row by citeable ID.
    assert!(
        err.contains(PUBLIC_NO_KEY_ROW_ID),
        "coverage error should name deleted row; got: {err}"
    );
}
