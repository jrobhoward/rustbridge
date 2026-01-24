//! Example message types for code generation demonstration.
//!
//! This file demonstrates the types of Rust structs that can be used
//! with rustbridge code generation.
//!
//! Generate JSON Schema:
//! ```bash
//! rustbridge generate json-schema -i examples/codegen/messages.rs -o examples/codegen/schema.json
//! ```
//!
//! Generate Java classes:
//! ```bash
//! rustbridge generate java -i examples/codegen/messages.rs -o examples/codegen/java -p com.example.messages
//! ```

use serde::{Deserialize, Serialize};

/// A simple greeting request.
///
/// This demonstrates basic string fields and documentation preservation.
#[derive(Serialize, Deserialize)]
pub struct GreetingRequest {
    /// The name of the person to greet.
    pub name: String,

    /// The greeting language code (e.g., "en", "es", "fr").
    ///
    /// If not provided, defaults to English.
    pub language: Option<String>,
}

/// Response to a greeting request.
#[derive(Serialize, Deserialize)]
pub struct GreetingResponse {
    /// The generated greeting message.
    pub message: String,

    /// Timestamp when the greeting was created (Unix epoch milliseconds).
    pub timestamp: i64,
}

/// User profile information.
///
/// Demonstrates various field types including optional fields,
/// custom types, and collections.
#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique user identifier.
    pub id: u64,

    /// User's display name.
    ///
    /// This field uses snake_case in Rust but will be converted
    /// to camelCase in Java: displayName
    pub display_name: String,

    /// User's email address (optional).
    pub email: Option<String>,

    /// User's age in years (optional).
    pub age: Option<u32>,

    /// List of tags associated with the user.
    pub tags: Vec<String>,

    /// User's account settings.
    pub settings: AccountSettings,
}

/// User account settings.
///
/// Demonstrates nested custom types.
#[derive(Serialize, Deserialize)]
pub struct AccountSettings {
    /// Enable email notifications.
    pub email_notifications: bool,

    /// Enable SMS notifications.
    pub sms_notifications: bool,

    /// Preferred language code.
    pub language: String,

    /// Preferred timezone (IANA timezone identifier).
    pub timezone: String,
}

/// A product in an e-commerce system.
///
/// Demonstrates numeric types and serde rename.
#[derive(Serialize, Deserialize)]
pub struct Product {
    /// Product SKU (Stock Keeping Unit).
    #[serde(rename = "sku")]
    pub stock_keeping_unit: String,

    /// Product name.
    pub name: String,

    /// Product description (optional).
    pub description: Option<String>,

    /// Price in cents (to avoid floating point precision issues).
    pub price_cents: u64,

    /// Available quantity.
    pub quantity: u32,

    /// Product categories.
    pub categories: Vec<String>,

    /// Product ratings.
    pub ratings: Vec<ProductRating>,
}

/// A product rating.
#[derive(Serialize, Deserialize)]
pub struct ProductRating {
    /// User who submitted the rating.
    pub user_id: u64,

    /// Rating value (1-5 stars).
    pub stars: u8,

    /// Rating comment (optional).
    pub comment: Option<String>,

    /// Rating timestamp (Unix epoch milliseconds).
    pub timestamp: i64,
}

/// Order information.
///
/// Demonstrates complex nested structures with multiple custom types.
#[derive(Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier.
    pub order_id: u64,

    /// Customer user ID.
    pub customer_id: u64,

    /// Order items.
    pub items: Vec<OrderItem>,

    /// Shipping address.
    pub shipping_address: Address,

    /// Billing address (optional, may be same as shipping).
    pub billing_address: Option<Address>,

    /// Order total in cents.
    pub total_cents: u64,

    /// Order status.
    pub status: String,

    /// Order creation timestamp (Unix epoch milliseconds).
    pub created_at: i64,
}

/// An item in an order.
#[derive(Serialize, Deserialize)]
pub struct OrderItem {
    /// Product SKU.
    pub sku: String,

    /// Quantity ordered.
    pub quantity: u32,

    /// Price per unit in cents (at time of order).
    pub unit_price_cents: u64,
}

/// A mailing address.
#[derive(Serialize, Deserialize)]
pub struct Address {
    /// Street address line 1.
    pub street1: String,

    /// Street address line 2 (optional).
    pub street2: Option<String>,

    /// City name.
    pub city: String,

    /// State/province code.
    pub state: String,

    /// Postal/ZIP code.
    pub postal_code: String,

    /// Country code (ISO 3166-1 alpha-2).
    pub country: String,
}

/// Search query and filters.
///
/// Demonstrates optional complex types.
#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    /// Search query string.
    pub query: String,

    /// Maximum number of results to return.
    pub limit: u32,

    /// Offset for pagination.
    pub offset: u32,

    /// Categories to filter by (optional).
    pub categories: Option<Vec<String>>,

    /// Minimum price in cents (optional).
    pub min_price_cents: Option<u64>,

    /// Maximum price in cents (optional).
    pub max_price_cents: Option<u64>,
}

/// Search results.
#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    /// Total number of results available.
    pub total: u64,

    /// Results for this page.
    pub results: Vec<Product>,

    /// Query execution time in milliseconds.
    pub query_time_ms: u32,
}
