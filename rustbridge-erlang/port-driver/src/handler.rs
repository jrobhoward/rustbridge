//! Command handler that dispatches protocol commands to `rustbridge-consumer`.

use std::sync::Arc;
use std::sync::mpsc::Sender;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rustbridge_consumer::{LogCallbackFn, NativePluginLoader};
use rustbridge_core::{LogLevel, PluginConfig};

use crate::error::PortError;
use crate::protocol::{Command, LogMessage, Response};

/// Manages a single plugin instance and dispatches commands to it.
pub struct Handler {
    plugin: Option<rustbridge_consumer::NativePlugin>,
    log_tx: Sender<LogMessage>,
}

impl Handler {
    pub fn new(log_tx: Sender<LogMessage>) -> Self {
        Self {
            plugin: None,
            log_tx,
        }
    }

    /// Dispatch a command and return a response.
    pub fn dispatch(&mut self, command: Command) -> Response {
        let id = command.id();
        match self.dispatch_inner(command) {
            Ok(resp) => resp,
            Err(e) => Response::error(id, e.code(), e.to_string()),
        }
    }

    fn dispatch_inner(&mut self, command: Command) -> Result<Response, PortError> {
        match command {
            Command::Load { id, path, config } => self.handle_load(id, &path, config),
            Command::LoadBundle {
                id,
                path,
                config,
                verify_signatures,
                public_key,
            } => self.handle_load_bundle(id, &path, config, verify_signatures, public_key),
            Command::Call {
                id,
                type_tag,
                payload,
            } => self.handle_call(id, &type_tag, &payload),
            Command::CallRaw {
                id,
                message_id,
                data,
            } => self.handle_call_raw(id, message_id, &data),
            Command::GetState { id } => self.handle_get_state(id),
            Command::GetRejectedCount { id } => self.handle_get_rejected_count(id),
            Command::SetLogLevel { id, level } => self.handle_set_log_level(id, level),
            Command::Shutdown { id } => self.handle_shutdown(id),
        }
    }

    fn handle_load(
        &mut self,
        id: u64,
        path: &str,
        config_value: Option<serde_json::Value>,
    ) -> Result<Response, PortError> {
        if self.plugin.is_some() {
            return Err(PortError::PluginAlreadyLoaded);
        }

        let config = parse_config(config_value)?;
        let log_callback = self.make_log_callback();

        let plugin = NativePluginLoader::load_with_config(path, &config, Some(log_callback))?;
        self.plugin = Some(plugin);

        Ok(Response::ok(id, serde_json::json!(true)))
    }

    fn handle_load_bundle(
        &mut self,
        id: u64,
        path: &str,
        config_value: Option<serde_json::Value>,
        verify_signatures: Option<bool>,
        public_key: Option<String>,
    ) -> Result<Response, PortError> {
        if self.plugin.is_some() {
            return Err(PortError::PluginAlreadyLoaded);
        }

        let config = parse_config(config_value)?;
        let log_callback = self.make_log_callback();
        let verify = verify_signatures.unwrap_or(false);

        let plugin = if verify {
            NativePluginLoader::load_bundle_with_verification(
                path,
                &config,
                Some(log_callback),
                true,
                public_key.as_deref(),
            )?
        } else {
            NativePluginLoader::load_bundle_with_config(path, &config, Some(log_callback))?
        };

        self.plugin = Some(plugin);

        Ok(Response::ok(id, serde_json::json!(true)))
    }

    fn handle_call(
        &mut self,
        id: u64,
        type_tag: &str,
        payload: &str,
    ) -> Result<Response, PortError> {
        let plugin = self.plugin.as_ref().ok_or(PortError::PluginNotLoaded)?;

        let result = plugin.call(type_tag, payload)?;

        Ok(Response::ok(id, serde_json::Value::String(result)))
    }

    fn handle_call_raw(
        &mut self,
        id: u64,
        message_id: u32,
        data_b64: &str,
    ) -> Result<Response, PortError> {
        let plugin = self.plugin.as_ref().ok_or(PortError::PluginNotLoaded)?;

        let request_bytes = BASE64
            .decode(data_b64)
            .map_err(|e| PortError::Base64Decode(e.to_string()))?;

        let response_bytes = plugin.call_raw(message_id, &request_bytes)?;
        let response_b64 = BASE64.encode(&response_bytes);

        Ok(Response::ok(id, serde_json::Value::String(response_b64)))
    }

    fn handle_get_state(&mut self, id: u64) -> Result<Response, PortError> {
        let plugin = self.plugin.as_ref().ok_or(PortError::PluginNotLoaded)?;

        let state = plugin.state();
        let state_str = format!("{state:?}").to_lowercase();

        Ok(Response::ok(id, serde_json::Value::String(state_str)))
    }

    fn handle_get_rejected_count(&mut self, id: u64) -> Result<Response, PortError> {
        let plugin = self.plugin.as_ref().ok_or(PortError::PluginNotLoaded)?;

        let count = plugin.rejected_request_count();

        Ok(Response::ok(id, serde_json::json!(count)))
    }

    fn handle_set_log_level(&mut self, id: u64, level: u8) -> Result<Response, PortError> {
        let plugin = self.plugin.as_ref().ok_or(PortError::PluginNotLoaded)?;

        plugin.set_log_level(LogLevel::from_u8(level));

        Ok(Response::ok(id, serde_json::json!(true)))
    }

    fn handle_shutdown(&mut self, id: u64) -> Result<Response, PortError> {
        let plugin = self.plugin.take().ok_or(PortError::PluginNotLoaded)?;

        plugin.shutdown()?;

        Ok(Response::ok(id, serde_json::json!(true)))
    }

    /// Create a log callback that sends log messages through the channel.
    fn make_log_callback(&self) -> LogCallbackFn {
        let tx = self.log_tx.clone();
        Arc::new(move |level: LogLevel, target: &str, message: &str| {
            let log_msg = LogMessage::new(level as u8, target.to_string(), message.to_string());
            // Ignore send errors (receiver dropped means we're shutting down)
            let _ = tx.send(log_msg);
        })
    }
}

/// Parse an optional JSON value into a PluginConfig.
fn parse_config(value: Option<serde_json::Value>) -> Result<PluginConfig, PortError> {
    match value {
        Some(v) => serde_json::from_value(v)
            .map_err(|e| PortError::Protocol(format!("invalid config: {e}"))),
        None => Ok(PluginConfig::default()),
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn make_handler() -> Handler {
        let (tx, _rx) = std::sync::mpsc::channel();
        Handler::new(tx)
    }

    #[test]
    fn dispatch___call_without_load___returns_plugin_not_loaded() {
        let mut handler = make_handler();
        let cmd = Command::Call {
            id: 1,
            type_tag: "echo".to_string(),
            payload: "{}".to_string(),
        };

        let resp = handler.dispatch(cmd);

        assert_eq!(resp.status, "error");
        assert_eq!(resp.error_code, Some(201));
    }

    #[test]
    fn dispatch___get_state_without_load___returns_plugin_not_loaded() {
        let mut handler = make_handler();
        let cmd = Command::GetState { id: 1 };

        let resp = handler.dispatch(cmd);

        assert_eq!(resp.status, "error");
        assert_eq!(resp.error_code, Some(201));
    }

    #[test]
    fn dispatch___get_rejected_count_without_load___returns_plugin_not_loaded() {
        let mut handler = make_handler();
        let cmd = Command::GetRejectedCount { id: 1 };

        let resp = handler.dispatch(cmd);

        assert_eq!(resp.status, "error");
        assert_eq!(resp.error_code, Some(201));
    }

    #[test]
    fn dispatch___shutdown_without_load___returns_plugin_not_loaded() {
        let mut handler = make_handler();
        let cmd = Command::Shutdown { id: 1 };

        let resp = handler.dispatch(cmd);

        assert_eq!(resp.status, "error");
        assert_eq!(resp.error_code, Some(201));
    }

    #[test]
    fn dispatch___load_nonexistent___returns_error() {
        let mut handler = make_handler();
        let cmd = Command::Load {
            id: 1,
            path: "/nonexistent/libplugin.so".to_string(),
            config: None,
        };

        let resp = handler.dispatch(cmd);

        assert_eq!(resp.status, "error");
    }

    #[test]
    fn parse_config___none___returns_defaults() {
        let config = parse_config(None).unwrap();

        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.max_concurrent_ops, 1000);
    }

    #[test]
    fn parse_config___valid_json___parses_correctly() {
        let value = serde_json::json!({"log_level": "debug", "max_concurrent_ops": 500});

        let config = parse_config(Some(value)).unwrap();

        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.max_concurrent_ops, 500);
    }

    #[test]
    fn parse_config___invalid_json___returns_error() {
        let value = serde_json::json!({"log_level": "not_a_level"});

        let result = parse_config(Some(value));

        assert!(result.is_err());
    }
}
