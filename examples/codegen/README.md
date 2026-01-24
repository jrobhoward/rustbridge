# Code Generation Examples

This directory contains example Rust message types and demonstrates how to use rustbridge's code generation features.

## Files

- **`messages.rs`**: Example Rust structs demonstrating various supported types and patterns
- **`schema.json`**: Generated JSON Schema (run generation commands to create)
- **`java/`**: Generated Java classes (run generation commands to create)

## Quick Start

### Generate JSON Schema

```bash
rustbridge generate json-schema \
  -i examples/codegen/messages.rs \
  -o examples/codegen/schema.json
```

This creates a JSON Schema file containing definitions for all message types.

### Generate Java Classes

```bash
rustbridge generate java \
  -i examples/codegen/messages.rs \
  -o examples/codegen/java \
  -p com.example.messages
```

This creates Java POJO classes in the `java/com/example/messages/` directory.

### View Generated Files

**JSON Schema:**
```bash
cat examples/codegen/schema.json | jq .
```

**Java Classes:**
```bash
ls -la examples/codegen/java/com/example/messages/
cat examples/codegen/java/com/example/messages/UserProfile.java
```

## Example Types

The `messages.rs` file demonstrates:

### Basic Types
- **GreetingRequest**: Simple string fields with optional language
- **GreetingResponse**: Response with message and timestamp

### Complex Types
- **UserProfile**: Multiple field types including optional fields, collections, and nested types
- **AccountSettings**: Nested custom type with boolean and string fields

### E-commerce Example
- **Product**: Demonstrates numeric types and serde rename
- **ProductRating**: Nested type with ratings
- **Order**: Complex order with items and addresses
- **OrderItem**: Line item in an order
- **Address**: Mailing address with optional fields

### Search Example
- **SearchRequest**: Query with optional filters
- **SearchResponse**: Results with pagination metadata

## Type Support

### Primitives
- `String`, `bool`
- `i8`, `i16`, `i32`, `i64`
- `u8`, `u16`, `u32`, `u64`
- `f32`, `f64`

### Containers
- `Vec<T>` - Arrays/Lists
- `Option<T>` - Optional fields

### Custom Types
- Nested structs
- Vec of custom types

## Naming Conventions

### Snake Case → Camel Case

Rust's `snake_case` is automatically converted to Java's `camelCase`:

```rust
pub struct UserProfile {
    pub display_name: String,  // Rust: display_name
}
```

```java
public class UserProfile {
    @SerializedName("display_name")
    public String displayName;  // Java: displayName
}
```

### Serde Rename

The `#[serde(rename = "...")]` attribute is preserved:

```rust
#[serde(rename = "sku")]
pub stock_keeping_unit: String,  // JSON: "sku", Java: stockKeepingUnit
```

## Documentation Preservation

Doc comments (`///`) are preserved in generated code:

**Rust:**
```rust
/// User profile information.
#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique user identifier.
    pub id: u64,
}
```

**JSON Schema:**
```json
{
  "UserProfile": {
    "description": "User profile information.",
    "properties": {
      "id": {
        "description": "Unique user identifier.",
        "type": "integer"
      }
    }
  }
}
```

**Java:**
```java
/**
 * User profile information.
 */
public class UserProfile {
    /**
     * Unique user identifier.
     */
    public long id;
}
```

## Integration with Bundles

You can auto-generate and embed schemas during bundle creation:

```bash
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --generate-schema examples/codegen/messages.rs:messages.json
```

The schema will be embedded in the bundle and can be extracted by consumers.

## See Also

- [Code Generation Guide](../../docs/CODE_GENERATION.md) - Comprehensive documentation
- [CLAUDE.md](../../CLAUDE.md) - Project overview and CLI usage
- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - System architecture
