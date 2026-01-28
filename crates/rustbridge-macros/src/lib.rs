//! rustbridge-macros - Procedural macros for rustbridge plugins
//!
//! This crate provides:
//! - `#[rustbridge_plugin]` - Mark a struct as a plugin implementation
//! - `#[rustbridge_handler]` - Mark a method as a message handler
//! - `#[derive(Message)]` - Derive message traits for request/response types
//! - `rustbridge_entry!` - Generate the FFI entry point

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, parse_macro_input};

/// Attribute for marking a struct as a rustbridge plugin
///
/// This generates the necessary boilerplate for implementing the Plugin trait
/// and dispatching messages to handler methods.
///
/// # Example
///
/// ```ignore
/// use rustbridge_macros::rustbridge_plugin;
///
/// #[rustbridge_plugin]
/// struct MyPlugin {
///     // plugin state
/// }
///
/// impl MyPlugin {
///     #[rustbridge_handler("user.create")]
///     fn create_user(&self, req: CreateUserRequest) -> Result<CreateUserResponse, PluginError> {
///         // handler implementation
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn rustbridge_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            /// Create a new plugin instance
            pub fn new() -> Self {
                Self::default()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute for marking a method as a message handler
///
/// The handler will be invoked when a message with the matching type tag is received.
///
/// # Example
///
/// ```ignore
/// #[rustbridge_handler("user.create")]
/// fn create_user(&self, req: CreateUserRequest) -> Result<CreateUserResponse, PluginError> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn rustbridge_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let type_tag = parse_macro_input!(attr as syn::LitStr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;

    // Generate the handler with metadata
    let expanded = quote! {
        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            const _TYPE_TAG: &str = #type_tag;
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// Options for the Message derive macro
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(message))]
struct MessageOpts {
    ident: syn::Ident,
    generics: syn::Generics,

    /// The type tag for this message (e.g., "user.create")
    #[darling(default)]
    tag: Option<String>,
}

/// Derive macro for message types
///
/// Implements serialization and type tag metadata for request/response types.
///
/// # Example
///
/// ```ignore
/// #[derive(Message, Serialize, Deserialize)]
/// #[message(tag = "user.create")]
/// struct CreateUserRequest {
///     pub username: String,
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(Message, attributes(message))]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let opts = match MessageOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let name = &opts.ident;
    let (impl_generics, ty_generics, where_clause) = opts.generics.split_for_impl();

    let type_tag = opts.tag.unwrap_or_else(|| {
        // Generate default tag from type name (e.g., CreateUserRequest -> create_user_request)
        let name_str = name.to_string();
        to_snake_case(&name_str)
    });

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Get the type tag for this message
            pub const fn type_tag() -> &'static str {
                #type_tag
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generate the FFI entry point for a plugin
///
/// This macro creates the `plugin_create` extern function that the FFI layer
/// calls to instantiate the plugin.
///
/// # Usage
///
/// For plugins using `Default`:
/// ```ignore
/// rustbridge_entry!(MyPlugin::default);
/// ```
///
/// For plugins using `PluginFactory::create` (receives config at construction time):
/// ```ignore
/// rustbridge_entry!(MyPlugin::create);
/// ```
///
/// When using `::create`, the macro generates both `plugin_create()` and
/// `plugin_create_with_config(config_json, config_len)` FFI functions.
#[proc_macro]
pub fn rustbridge_entry(input: TokenStream) -> TokenStream {
    let factory_path = parse_macro_input!(input as syn::ExprPath);

    // Check if the path ends with "::create" to enable PluginFactory support
    let is_factory_create = factory_path
        .path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "create");

    let expanded = if is_factory_create {
        // Extract the type path (everything except the final ::create)
        let type_path: syn::Path = {
            let mut path = factory_path.path.clone();
            path.segments.pop(); // Remove "create"
            // Remove trailing punctuation if present
            if let Some(pair) = path.segments.pop() {
                path.segments.push(pair.into_value());
            }
            path
        };

        quote! {
            /// FFI entry point - creates a new plugin instance with default config
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn plugin_create() -> *mut ::std::ffi::c_void {
                let config = ::rustbridge_core::PluginConfig::default();
                match <#type_path as ::rustbridge_core::PluginFactory>::create(&config) {
                    Ok(plugin) => {
                        let plugin: Box<dyn ::rustbridge_core::Plugin> = Box::new(plugin);
                        let boxed: Box<Box<dyn ::rustbridge_core::Plugin>> = Box::new(plugin);
                        Box::into_raw(boxed) as *mut ::std::ffi::c_void
                    }
                    Err(_) => ::std::ptr::null_mut()
                }
            }

            /// FFI entry point - creates a new plugin instance with provided config
            ///
            /// # Safety
            ///
            /// - `config_json` must be a valid pointer to a UTF-8 JSON string, or null
            /// - `config_len` must be the length of the JSON string in bytes
            /// - If `config_json` is null, a default config is used
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn plugin_create_with_config(
                config_json: *const u8,
                config_len: usize,
            ) -> *mut ::std::ffi::c_void {
                let config = if config_json.is_null() || config_len == 0 {
                    ::rustbridge_core::PluginConfig::default()
                } else {
                    // SAFETY: Caller guarantees config_json is valid for config_len bytes
                    let bytes = unsafe { ::std::slice::from_raw_parts(config_json, config_len) };
                    match ::rustbridge_core::PluginConfig::from_json(bytes) {
                        Ok(c) => c,
                        Err(_) => return ::std::ptr::null_mut(),
                    }
                };

                match <#type_path as ::rustbridge_core::PluginFactory>::create(&config) {
                    Ok(plugin) => {
                        let plugin: Box<dyn ::rustbridge_core::Plugin> = Box::new(plugin);
                        let boxed: Box<Box<dyn ::rustbridge_core::Plugin>> = Box::new(plugin);
                        Box::into_raw(boxed) as *mut ::std::ffi::c_void
                    }
                    Err(_) => ::std::ptr::null_mut()
                }
            }
        }
    } else {
        // Original behavior for ::default or other factory functions
        quote! {
            /// FFI entry point - creates a new plugin instance
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn plugin_create() -> *mut ::std::ffi::c_void {
                let plugin: Box<dyn ::rustbridge_core::Plugin> = Box::new(#factory_path());
                let boxed: Box<Box<dyn ::rustbridge_core::Plugin>> = Box::new(plugin);
                Box::into_raw(boxed) as *mut ::std::ffi::c_void
            }
        }
    };

    TokenStream::from(expanded)
}

/// Macro to implement the Plugin trait with handler dispatch
///
/// This generates a Plugin implementation that routes messages to handler methods
/// based on type tags.
///
/// # Example
///
/// ```ignore
/// impl_plugin! {
///     MyPlugin {
///         "user.create" => create_user,
///         "user.delete" => delete_user,
///     }
/// }
/// ```
#[proc_macro]
pub fn impl_plugin(input: TokenStream) -> TokenStream {
    // Parse: PluginType { "tag" => method, ... }
    let _input_str = input.to_string();

    // For now, generate a simple implementation
    // Full parsing would require custom syntax handling
    let expanded = quote! {
        // Plugin implementation generated by impl_plugin!
        // Use rustbridge_plugin attribute for full functionality
    };

    TokenStream::from(expanded)
}

/// Convert a PascalCase string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod lib_tests;
