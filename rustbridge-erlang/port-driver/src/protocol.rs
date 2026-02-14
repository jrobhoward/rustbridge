//! Wire protocol types for Erlang port communication.
//!
//! All messages are JSON objects with a `"type"` discriminator field.
//! Framing uses Erlang's `{packet, 4}` convention: 4-byte big-endian length prefix.

use serde::{Deserialize, Serialize};

/// Commands sent from Erlang to the port driver.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    /// Load a plugin from a shared library path.
    #[serde(rename = "load")]
    Load {
        id: u64,
        path: String,
        #[serde(default)]
        config: Option<serde_json::Value>,
    },

    /// Load a plugin from a .rbp bundle.
    #[serde(rename = "load_bundle")]
    LoadBundle {
        id: u64,
        path: String,
        #[serde(default)]
        config: Option<serde_json::Value>,
        #[serde(default)]
        verify_signatures: Option<bool>,
        #[serde(default)]
        public_key: Option<String>,
    },

    /// Make a JSON transport call.
    #[serde(rename = "call")]
    Call {
        id: u64,
        type_tag: String,
        payload: String,
    },

    /// Make a binary transport call (data is base64-encoded).
    #[serde(rename = "call_raw")]
    CallRaw {
        id: u64,
        message_id: u32,
        data: String,
    },

    /// Query the plugin lifecycle state.
    #[serde(rename = "get_state")]
    GetState { id: u64 },

    /// Set the plugin log level.
    #[serde(rename = "set_log_level")]
    SetLogLevel { id: u64, level: u8 },

    /// Shutdown the plugin.
    #[serde(rename = "shutdown")]
    Shutdown { id: u64 },
}

impl Command {
    /// Get the request id for correlation.
    pub fn id(&self) -> u64 {
        match self {
            Command::Load { id, .. }
            | Command::LoadBundle { id, .. }
            | Command::Call { id, .. }
            | Command::CallRaw { id, .. }
            | Command::GetState { id }
            | Command::SetLogLevel { id, .. }
            | Command::Shutdown { id } => *id,
        }
    }
}

/// Response sent from the port driver back to Erlang.
#[derive(Debug, Serialize)]
pub struct Response {
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    pub id: u64,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl Response {
    /// Create a success response.
    pub fn ok(id: u64, data: serde_json::Value) -> Self {
        Self {
            msg_type: "response",
            id,
            status: "ok",
            data: Some(data),
            error_code: None,
            error_message: None,
        }
    }

    /// Create an error response.
    pub fn error(id: u64, code: u32, message: String) -> Self {
        Self {
            msg_type: "response",
            id,
            status: "error",
            data: None,
            error_code: Some(code),
            error_message: Some(message),
        }
    }
}

/// Asynchronous log message sent from the port driver to Erlang.
#[derive(Debug, Serialize)]
pub struct LogMessage {
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    pub level: u8,
    pub target: String,
    pub message: String,
}

impl LogMessage {
    pub fn new(level: u8, target: String, message: String) -> Self {
        Self {
            msg_type: "log",
            level,
            target,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn Command___load___deserializes_correctly() {
        let json = r#"{"type": "load", "id": 1, "path": "/tmp/libplugin.so"}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 1);
        assert!(matches!(cmd, Command::Load { path, .. } if path == "/tmp/libplugin.so"));
    }

    #[test]
    fn Command___load_with_config___deserializes_correctly() {
        let json =
            r#"{"type": "load", "id": 2, "path": "/tmp/lib.so", "config": {"log_level": "debug"}}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 2);
        if let Command::Load { config, .. } = cmd {
            assert!(config.is_some());
        } else {
            panic!("expected Load command");
        }
    }

    #[test]
    fn Command___call___deserializes_correctly() {
        let json = r#"{"type": "call", "id": 3, "type_tag": "echo", "payload": "{\"message\": \"hello\"}"}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 3);
        assert!(matches!(cmd, Command::Call { type_tag, .. } if type_tag == "echo"));
    }

    #[test]
    fn Command___call_raw___deserializes_correctly() {
        let json = r#"{"type": "call_raw", "id": 4, "message_id": 1, "data": "AQID"}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 4);
        assert!(matches!(cmd, Command::CallRaw { message_id: 1, .. }));
    }

    #[test]
    fn Command___get_state___deserializes_correctly() {
        let json = r#"{"type": "get_state", "id": 5}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 5);
        assert!(matches!(cmd, Command::GetState { .. }));
    }

    #[test]
    fn Command___set_log_level___deserializes_correctly() {
        let json = r#"{"type": "set_log_level", "id": 6, "level": 3}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 6);
        assert!(matches!(cmd, Command::SetLogLevel { level: 3, .. }));
    }

    #[test]
    fn Command___shutdown___deserializes_correctly() {
        let json = r#"{"type": "shutdown", "id": 7}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 7);
        assert!(matches!(cmd, Command::Shutdown { .. }));
    }

    #[test]
    fn Command___load_bundle___deserializes_correctly() {
        let json = r#"{"type": "load_bundle", "id": 8, "path": "/tmp/plugin.rbp", "verify_signatures": true}"#;

        let cmd: Command = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.id(), 8);
        assert!(matches!(
            cmd,
            Command::LoadBundle {
                verify_signatures: Some(true),
                ..
            }
        ));
    }

    #[test]
    fn Response___ok___serializes_correctly() {
        let resp = Response::ok(1, serde_json::json!("active"));

        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains(r#""type":"response""#));
        assert!(json.contains(r#""id":1"#));
        assert!(json.contains(r#""status":"ok""#));
        assert!(json.contains(r#""data":"active""#));
        assert!(!json.contains("error_code"));
    }

    #[test]
    fn Response___error___serializes_correctly() {
        let resp = Response::error(2, 6, "unknown message type".to_string());

        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains(r#""status":"error""#));
        assert!(json.contains(r#""error_code":6"#));
        assert!(json.contains("unknown message type"));
        assert!(!json.contains(r#""data""#));
    }

    #[test]
    fn LogMessage___new___serializes_correctly() {
        let log = LogMessage::new(2, "hello_plugin".to_string(), "starting up".to_string());

        let json = serde_json::to_string(&log).unwrap();

        assert!(json.contains(r#""type":"log""#));
        assert!(json.contains(r#""level":2"#));
        assert!(json.contains(r#""target":"hello_plugin""#));
        assert!(json.contains(r#""message":"starting up""#));
    }
}
