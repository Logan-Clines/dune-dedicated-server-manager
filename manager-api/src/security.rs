use serde_json::Value;

pub fn redact_json(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, child)| {
                    let lower = key.to_ascii_lowercase();
                    if contains_sensitive_marker(&lower) {
                        (key, Value::String("<redacted>".to_string()))
                    } else {
                        (key, redact_json(child))
                    }
                })
                .collect::<serde_json::Map<_, _>>(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(redact_json).collect()),
        Value::String(text)
            if contains_sensitive_marker(&text.to_ascii_lowercase())
                || contains_jwt_like_segment(&text) =>
        {
            Value::String("<redacted>".to_string())
        }
        other => other,
    }
}

pub fn redact_text(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if contains_sensitive_marker(&lower) || contains_jwt_like_segment(line) {
                "<redacted>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn contains_sensitive_marker(value: &str) -> bool {
    value.contains("token")
        || value.contains("secret")
        || value.contains("password")
        || value.contains("apikey")
        || value.contains("auth")
}

fn contains_jwt_like_segment(value: &str) -> bool {
    value
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')))
        .any(looks_like_jwt)
}

fn looks_like_jwt(value: &str) -> bool {
    let mut parts = value.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(a), Some(b), Some(c), None) if a.len() > 8 && b.len() > 8 && c.len() > 8
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{redact_json, redact_text};

    #[test]
    fn redacts_sensitive_strings_inside_arrays() {
        let redacted = redact_json(json!({
            "arguments": [
                "-FarmRegion=Europe Test",
                "-ini:engine:[FuncomLiveServices]:ServiceAuthToken=eyJhbGciOiJIUzI1NiJ9.eyJIb3N0SWQiOiJ0ZXN0In0.abcdefghi"
            ]
        }));

        assert_eq!(redacted["arguments"][0], "-FarmRegion=Europe Test");
        assert_eq!(redacted["arguments"][1], "<redacted>");
    }

    #[test]
    fn redacts_sensitive_text_lines() {
        let text = "ok\nServiceAuthToken=secret-value";
        assert_eq!(redact_text(text), "ok\n<redacted>");
    }
}
