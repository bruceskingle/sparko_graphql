/// Hiding secrets such as passwords, API keys and tokens in verbose output.

/// What a secret is replaced with.
pub const REDACTED: &str = "<redacted>";

/// Parts of names, in lower case without separators, which show that a value is a secret.
const SECRET_NAMES: [&str; 5] = ["password", "token", "secret", "apikey", "presignedkey"];

/// Whether a field or argument with this name holds a secret, e.g. password, apiKey, api_key, refreshToken.
pub fn is_secret_name(name: &str) -> bool {
    let name: String = name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    SECRET_NAMES.iter().any(|secret| name.contains(secret))
}

/// Replace the value of every field with a secret name, at any depth, except nulls which show that there is no value.
pub fn redact_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (name, field) in map.iter_mut() {
                if is_secret_name(name) {
                    if !field.is_null() {
                        *field = serde_json::Value::String(String::from(REDACTED));
                    }
                }
                else {
                    redact_secrets(field);
                }
            }
        },
        serde_json::Value::Array(items) => items.iter_mut().for_each(redact_secrets),
        _ => {},
    }
}

/// JSON text with its secrets redacted, pretty printed. Text which is not JSON is redacted entirely, as it may
/// contain secrets which cannot be found.
pub fn redacted_json(json: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(mut value) => {
            redact_secrets(&mut value);
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| String::from(REDACTED))
        },
        Err(_) => String::from(REDACTED),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_names_are_found_in_any_case_or_style() {
        for name in ["password", "APIKey", "api_key", "octopus-api-key", "refreshToken", "token", "organizationSecretKey", "preSignedKey"] {
            assert!(is_secret_name(name), "{name}");
        }
        for name in ["email", "accountNumber", "refreshExpiresIn", "endCursor", "hashKey"] {
            assert!(!is_secret_name(name), "{name}");
        }
    }

    #[test]
    fn secrets_are_redacted_at_any_depth_but_nulls_are_kept() {
        let mut value = serde_json::json!({
            "input": {"email": "a@b.c", "password": "pw", "APIKey": null},
            "list": [{"token": "t"}],
        });
        redact_secrets(&mut value);
        assert_eq!(value, serde_json::json!({
            "input": {"email": "a@b.c", "password": REDACTED, "APIKey": null},
            "list": [{"token": REDACTED}],
        }));
    }

    #[test]
    fn text_which_is_not_json_is_redacted() {
        assert_eq!(redacted_json("password=pw"), REDACTED);
    }
}
