//! JSON printing, the response envelope, and the stderr error report.

use serde::Serialize;

/// One machine-readable error inside an [`Envelope`] or error report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorEntry {
    pub code: String,
    pub message: String,
}

impl ErrorEntry {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// The shared response envelope:
///
/// ```json
/// {"tool": "<name>", "schema_version": <n>, "ok": bool, "data": ..., "errors": [{"code","message"}]}
/// ```
///
/// Tools with a pre-existing pinned JSON contract keep it and use
/// [`print_raw_json`] instead; see the README for who is on the envelope.
#[derive(Debug, Clone, Serialize)]
pub struct Envelope<T> {
    pub tool: String,
    pub schema_version: u32,
    pub ok: bool,
    pub data: T,
    pub errors: Vec<ErrorEntry>,
}

impl<T: Serialize> Envelope<T> {
    /// A successful response carrying `data`.
    pub fn ok(tool: &str, schema_version: u32, data: T) -> Self {
        Self {
            tool: tool.to_string(),
            schema_version,
            ok: true,
            data,
            errors: Vec::new(),
        }
    }

    /// A failed response; `data` may still carry partial results.
    pub fn failed(tool: &str, schema_version: u32, data: T, errors: Vec<ErrorEntry>) -> Self {
        Self {
            tool: tool.to_string(),
            schema_version,
            ok: false,
            data,
            errors,
        }
    }
}

/// Pretty-print an [`Envelope`] to stdout.
pub fn print_json<T: Serialize>(envelope: &Envelope<T>) -> serde_json::Result<()> {
    print_raw_json(envelope)
}

/// Pretty-print any serializable value to stdout, unwrapped. This is what the
/// tools with a pinned JSON contract use.
pub fn print_raw_json<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// The legacy error shape every tool has printed on stderr in JSON mode:
/// `{"ok": false, "error": {"code": ..., "message": ...}}`.
pub fn error_value(code: &str, message: &str) -> serde_json::Value {
    serde_json::json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message,
        }
    })
}

/// Report a fatal error on stderr. In JSON mode this is [`error_value`]
/// pretty-printed; in text mode it is `error: <message>`.
pub fn report_error(is_json: bool, code: &str, message: &str) {
    if is_json {
        let value = error_value(code, message);
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!(
                "{{\"ok\":false,\"error\":{{\"message\":\"{message}\"}}}}"
            ))
        );
    } else {
        eprintln!("error: {message}");
    }
}

/// Report a fatal error on stderr for a tool that is on the envelope: a
/// failed [`Envelope`] with `data: null` in JSON mode, `error: <message>` in
/// text mode.
pub fn report_error_envelope(
    is_json: bool,
    tool: &str,
    schema_version: u32,
    code: &str,
    message: &str,
) {
    if is_json {
        let envelope = Envelope::failed(
            tool,
            schema_version,
            serde_json::Value::Null,
            vec![ErrorEntry::new(code, message)],
        );
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| format!(
                "{{\"tool\":\"{tool}\",\"schema_version\":{schema_version},\"ok\":false,\"data\":null,\"errors\":[{{\"code\":\"{code}\",\"message\":\"{message}\"}}]}}"
            ))
        );
    } else {
        eprintln!("error: {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ok_envelope_has_fixed_key_set_and_order() {
        let envelope = Envelope::ok("quarry", 1, json!({ "x": 1 }));
        let text = serde_json::to_string(&envelope).unwrap();
        assert_eq!(
            text,
            r#"{"tool":"quarry","schema_version":1,"ok":true,"data":{"x":1},"errors":[]}"#
        );
    }

    #[test]
    fn failed_envelope_carries_errors() {
        let envelope = Envelope::failed(
            "quarry",
            1,
            serde_json::Value::Null,
            vec![ErrorEntry::new("not_found", "no manifest")],
        );
        let value = serde_json::to_value(&envelope).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["data"], serde_json::Value::Null);
        assert_eq!(value["errors"][0]["code"], "not_found");
        assert_eq!(value["errors"][0]["message"], "no manifest");
    }

    #[test]
    fn legacy_error_value_shape() {
        let value = error_value("validation_error", "bad input");
        assert_eq!(
            value,
            json!({ "ok": false, "error": { "code": "validation_error", "message": "bad input" } })
        );
    }
}
