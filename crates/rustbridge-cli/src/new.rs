//! New project command implementation

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Run the new command
pub fn run(name: &str, path: Option<String>) -> Result<()> {
    let project_dir = path.unwrap_or_else(|| name.to_string());
    let project_path = Path::new(&project_dir);

    println!("Creating new rustbridge plugin: {}", name);
    println!("Directory: {}", project_dir);

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory already exists: {}", project_dir);
    }

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("schemas"))?;

    // Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.5.0"
edition = "2021"

[workspace]  # Standalone project (not part of a parent workspace)

[lib]
crate-type = ["cdylib"]

[dependencies]
# rustbridge dependencies (fetched from GitHub)
rustbridge-core = {{ git = "https://github.com/jrobhoward/rustbridge.git" }}
rustbridge-ffi = {{ git = "https://github.com/jrobhoward/rustbridge.git" }}
rustbridge-macros = {{ git = "https://github.com/jrobhoward/rustbridge.git" }}

serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
async-trait = "0.1"
tokio = {{ version = "1.35", features = ["full"] }}
tracing = "0.1"
"#,
        name = name,
    );
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

    // Generate rustbridge.toml
    let manifest = format!(
        r#"[plugin]
name = "{name}"
version = "0.5.0"
description = "A rustbridge plugin"

[messages."echo"]
description = "Echo the input"

[platforms]
linux-x86_64 = "lib{name}.so"
darwin-aarch64 = "lib{name}.dylib"
darwin-x86_64 = "lib{name}.dylib"
windows-x86_64 = "{name}.dll"
"#,
        name = name,
    );
    fs::write(project_path.join("rustbridge.toml"), manifest)?;

    // Generate src/lib.rs
    let lib_rs = format!(
        r#"//! {name} - A rustbridge plugin

use async_trait::async_trait;
use rustbridge_core::{{Plugin, PluginConfig, PluginContext, PluginError, PluginResult}};
use rustbridge_macros::{{rustbridge_entry, Message}};
use serde::{{Deserialize, Serialize}};

/// Echo request message
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {{
    pub message: String,
}}

/// Echo response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {{
    pub message: String,
    pub length: usize,
}}

/// Plugin implementation
#[derive(Default)]
pub struct {class_name};

#[async_trait]
impl Plugin for {class_name} {{
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {{
        tracing::info!("{name} plugin started");
        Ok(())
    }}

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {{
        match type_tag {{
            "echo" => {{
                let req: EchoRequest = serde_json::from_slice(payload)?;
                let response = EchoResponse {{
                    length: req.message.len(),
                    message: req.message,
                }};
                Ok(serde_json::to_vec(&response)?)
            }}
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }}
    }}

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {{
        tracing::info!("{name} plugin stopped");
        Ok(())
    }}

    fn supported_types(&self) -> Vec<&'static str> {{
        vec!["echo"]
    }}
}}

// Generate FFI entry point
rustbridge_entry!({class_name}::default);

// Re-export FFI functions for the compiled library
pub use rustbridge_ffi::{{
    plugin_call,
    plugin_free_buffer,
    plugin_get_rejected_count,
    plugin_get_state,
    plugin_init,
    plugin_set_log_level,
    plugin_shutdown,
}};

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_echo() {{
        let plugin = {class_name};
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&EchoRequest {{
            message: "Hello, World!".to_string(),
        }})
        .unwrap();

        let response = plugin.handle_request(&ctx, "echo", &request).await.unwrap();
        let echo_response: EchoResponse = serde_json::from_slice(&response).unwrap();

        assert_eq!(echo_response.message, "Hello, World!");
        assert_eq!(echo_response.length, 13);
    }}
}}
"#,
        name = name,
        class_name = to_pascal_case(name),
    );
    fs::write(project_path.join("src/lib.rs"), lib_rs)?;

    // Generate .gitignore
    let gitignore = r#"/target/
Cargo.lock
*.swp
*.swo
.idea/
.vscode/
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;

    println!("\n✓ Project created successfully!");
    println!("\nNext steps:");
    println!("  cd {}", project_dir);
    println!("  cargo build");
    println!("  rustbridge check");

    Ok(())
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
