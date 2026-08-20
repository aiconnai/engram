use super::*;

pub(crate) fn metadata_value_to_param(
    key: &str,
    value: &serde_json::Value,
    conditions: &mut Vec<String>,
    params: &mut Vec<Box<dyn rusqlite::ToSql>>,
) -> Result<()> {
    // Validate key: only safe identifier characters to prevent injection
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
    {
        return Err(EngramError::InvalidInput(format!(
            "Invalid metadata filter key: '{}'. Keys must contain only alphanumeric characters, underscores, dots, or hyphens.",
            key
        )));
    }
    let json_path = format!("$.{}", key);

    match value {
        serde_json::Value::String(s) => {
            conditions.push("json_extract(m.metadata, ?) = ?".to_string());
            params.push(Box::new(json_path));
            params.push(Box::new(s.clone()));
        }
        serde_json::Value::Number(n) => {
            conditions.push("json_extract(m.metadata, ?) = ?".to_string());
            params.push(Box::new(json_path));
            if let Some(i) = n.as_i64() {
                params.push(Box::new(i));
            } else if let Some(f) = n.as_f64() {
                params.push(Box::new(f));
            } else {
                return Err(EngramError::InvalidInput("Invalid number".to_string()));
            }
        }
        serde_json::Value::Bool(b) => {
            conditions.push("json_extract(m.metadata, ?) = ?".to_string());
            params.push(Box::new(json_path));
            params.push(Box::new(*b));
        }
        serde_json::Value::Null => {
            conditions.push("json_extract(m.metadata, ?) IS NULL".to_string());
            params.push(Box::new(json_path));
        }
        _ => {
            return Err(EngramError::InvalidInput(format!(
                "Unsupported metadata filter value for key: {}",
                key
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_key_injection_rejected() {
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let value = serde_json::Value::String("test".to_string());

        // Valid keys should pass
        assert!(metadata_value_to_param("valid_key", &value, &mut conditions, &mut params).is_ok());
        assert!(
            metadata_value_to_param("key.nested", &value, &mut conditions, &mut params).is_ok()
        );
        assert!(
            metadata_value_to_param("key-with-dash", &value, &mut conditions, &mut params).is_ok()
        );

        // SQL injection attempts should fail
        assert!(metadata_value_to_param(
            "foo') = 1 OR 1=1 --",
            &value,
            &mut conditions,
            &mut params
        )
        .is_err());
        assert!(
            metadata_value_to_param("key with spaces", &value, &mut conditions, &mut params)
                .is_err()
        );
        assert!(metadata_value_to_param("", &value, &mut conditions, &mut params).is_err());
        assert!(
            metadata_value_to_param("key;DROP TABLE", &value, &mut conditions, &mut params)
                .is_err()
        );
    }
}
