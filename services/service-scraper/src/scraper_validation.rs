use serde_json::Value;

#[derive(serde::Deserialize, Default)]
struct FieldRule {
    field: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    format: Option<String>,
    #[serde(rename = "type", default)]
    field_type: Option<String>,
    #[serde(default)]
    min_length: Option<i64>,
    #[serde(default)]
    min: Option<f64>,
    #[serde(default)]
    max: Option<f64>,
}

/// Validates raw_data against field_rules JSON.
/// Returns (status, errors) where status is "VALID" or "INVALID".
pub fn validate(raw_data: &Value, field_rules_json: &str) -> (String, Vec<String>) {
    let rules: Vec<FieldRule> = match serde_json::from_str(field_rules_json) {
        Ok(r) => r,
        Err(_) => return ("VALID".to_string(), vec![]),
    };

    if rules.is_empty() {
        return ("VALID".to_string(), vec![]);
    }

    let mut errors: Vec<String> = Vec::new();

    for rule in &rules {
        let value = raw_data.get(&rule.field);

        // Required check
        if rule.required {
            match value {
                None => {
                    errors.push(format!("Field '{}' is required", rule.field));
                    continue;
                }
                Some(Value::Null) => {
                    errors.push(format!("Field '{}' cannot be null", rule.field));
                    continue;
                }
                Some(Value::String(s)) if s.is_empty() => {
                    errors.push(format!("Field '{}' cannot be empty", rule.field));
                    continue;
                }
                _ => {}
            }
        }

        if let Some(val) = value {
            // min_length check (strings only)
            if let Some(min_len) = rule.min_length {
                if let Value::String(s) = val {
                    if (s.len() as i64) < min_len {
                        errors.push(format!(
                            "Field '{}' must be at least {} characters",
                            rule.field, min_len
                        ));
                    }
                }
            }

            // format check
            if let Some(format) = &rule.format {
                if let Value::String(s) = val {
                    match format.as_str() {
                        "url" => {
                            if !s.starts_with("http://") && !s.starts_with("https://") {
                                errors.push(format!(
                                    "Field '{}' must be a valid URL",
                                    rule.field
                                ));
                            }
                        }
                        "email" => {
                            if !s.contains('@') || !s.contains('.') {
                                errors.push(format!(
                                    "Field '{}' must be a valid email",
                                    rule.field
                                ));
                            }
                        }
                        "date_iso" => {
                            // Basic ISO 8601 check: must start with a 4-digit year
                            let valid = s.len() >= 10
                                && s[..4].chars().all(|c| c.is_ascii_digit())
                                && s.as_bytes().get(4) == Some(&b'-');
                            if !valid {
                                errors.push(format!(
                                    "Field '{}' must be a valid ISO date",
                                    rule.field
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }

            // type check
            if let Some(field_type) = &rule.field_type {
                match field_type.as_str() {
                    "number" => {
                        if !val.is_number() {
                            errors.push(format!("Field '{}' must be a number", rule.field));
                        } else {
                            let n = val.as_f64().unwrap_or(0.0);
                            if let Some(min) = rule.min {
                                if n < min {
                                    errors.push(format!(
                                        "Field '{}' must be >= {}",
                                        rule.field, min
                                    ));
                                }
                            }
                            if let Some(max) = rule.max {
                                if n > max {
                                    errors.push(format!(
                                        "Field '{}' must be <= {}",
                                        rule.field, max
                                    ));
                                }
                            }
                        }
                    }
                    "boolean" => {
                        if !val.is_boolean() {
                            errors.push(format!("Field '{}' must be a boolean", rule.field));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if errors.is_empty() {
        ("VALID".to_string(), vec![])
    } else {
        ("INVALID".to_string(), errors)
    }
}
