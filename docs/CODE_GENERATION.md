# Code Generation

This document describes rustbridge's code generation capabilities for creating language bindings from Rust message types.

## Overview

rustbridge can automatically generate code from Rust message types marked with `#[derive(Serialize, Deserialize)]` or `#[derive(Message)]`. This enables:

- **Self-documenting APIs**: Generate JSON schemas for API documentation
- **Type-safe bindings**: Generate Java/Kotlin classes with proper types and annotations
- **Reduced boilerplate**: Auto-generate POJOs, getters, setters, and serialization code
- **Consistency**: Single source of truth (Rust structs) for all language bindings

## Supported Generators

### 1. JSON Schema

Generates [JSON Schema Draft-07](https://json-schema.org/draft-07/schema) from Rust types.

**Use cases:**
- API documentation
- Contract testing
- Validation schemas
- OpenAPI/Swagger integration

**Example:**
```bash
rustbridge generate json-schema \
  -i src/messages.rs \
  -o schema/messages.json
```

### 2. Java Classes

Generates Java POJOs with Gson annotations for JSON serialization.

**Use cases:**
- Java/Kotlin client libraries
- Android apps
- JVM-based services consuming rustbridge plugins

**Example:**
```bash
rustbridge generate java \
  -i src/messages.rs \
  -o src/main/java \
  -p com.example.messages
```

## Supported Rust Types

### Primitives

| Rust Type | JSON Schema | Java Type (required) | Java Type (optional) |
|-----------|-------------|---------------------|---------------------|
| `String` | `string` | `String` | `String` |
| `bool` | `boolean` | `boolean` | `Boolean` |
| `i8`, `i16`, `i32` | `integer` | `int` | `Integer` |
| `i64` | `integer` | `long` | `Long` |
| `u8`, `u16`, `u32` | `integer` (min: 0) | `int` | `Integer` |
| `u64` | `integer` (min: 0) | `long` | `Long` |
| `f32` | `number` | `float` | `Float` |
| `f64` | `number` | `double` | `Double` |

### Container Types

- **`Vec<T>`**: Maps to JSON Schema `array` and Java `List<T>`
- **`Option<T>`**: Handled by marking fields as optional (not in `required` array)

### Custom Types

Custom struct types are supported and generate references:
- JSON Schema: `{"$ref": "#/definitions/CustomType"}`
- Java: Direct type reference

### Complex Examples

```rust
use serde::{Serialize, Deserialize};

/// User profile information.
#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique user ID.
    pub id: u64,

    /// User's display name.
    #[serde(rename = "displayName")]
    pub display_name: String,

    /// User's email address (optional).
    pub email: Option<String>,

    /// User's tags.
    pub tags: Vec<String>,

    /// User's preferences.
    pub preferences: UserPreferences,
}

/// User preferences.
#[derive(Serialize, Deserialize)]
pub struct UserPreferences {
    /// Enable email notifications.
    pub email_notifications: bool,

    /// Preferred language code.
    pub language: String,
}
```

## Generated Code Examples

### JSON Schema Output

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "UserProfile": {
      "type": "object",
      "description": "User profile information.",
      "properties": {
        "id": {
          "type": "integer",
          "minimum": 0,
          "description": "Unique user ID."
        },
        "displayName": {
          "type": "string",
          "description": "User's display name."
        },
        "email": {
          "type": "string",
          "description": "User's email address (optional)."
        },
        "tags": {
          "type": "array",
          "items": {"type": "string"},
          "description": "User's tags."
        },
        "preferences": {
          "$ref": "#/definitions/UserPreferences",
          "description": "User's preferences."
        }
      },
      "required": ["id", "displayName", "tags", "preferences"]
    },
    "UserPreferences": {
      "type": "object",
      "description": "User preferences.",
      "properties": {
        "email_notifications": {
          "type": "boolean",
          "description": "Enable email notifications."
        },
        "language": {
          "type": "string",
          "description": "Preferred language code."
        }
      },
      "required": ["email_notifications", "language"]
    }
  }
}
```

### Java Class Output

```java
package com.example.messages;

import com.google.gson.annotations.SerializedName;
import java.util.List;

/**
 * User profile information.
 */
public class UserProfile {

    /**
     * Unique user ID.
     */
    public long id;

    /**
     * User's display name.
     */
    @SerializedName("displayName")
    public String displayName;

    /**
     * User's email address (optional).
     */
    public String email;

    /**
     * User's tags.
     */
    public List<String> tags;

    /**
     * User's preferences.
     */
    public UserPreferences preferences;

    public UserProfile() {}

    public UserProfile(long id, String displayName, String email,
                       List<String> tags, UserPreferences preferences) {
        this.id = id;
        this.displayName = displayName;
        this.email = email;
        this.tags = tags;
        this.preferences = preferences;
    }

    public long getId() {
        return id;
    }

    public void setId(long id) {
        this.id = id;
    }

    // ... (other getters/setters)
}
```

## Integration with Bundles

Code generation can be integrated with bundle creation to automatically generate and embed schemas:

```bash
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --generate-schema src/messages.rs:messages.json \
  --generate-header src/binary_messages.rs:messages.h \
  --sign-key ~/.rustbridge/signing.key
```

This will:
1. Parse `src/messages.rs` for message types
2. Generate JSON schema
3. Embed the schema in the bundle at `schema/messages.json`
4. Calculate and store the schema checksum in the manifest

Consumers can then extract the schema from the bundle:

```java
BundleLoader loader = BundleLoader.builder()
    .bundlePath("my-plugin-1.0.0.rbp")
    .build();

// List all schemas
Map<String, SchemaInfo> schemas = loader.getSchemas();

// Read schema content
String jsonSchema = loader.readSchema("messages.json");

// Or extract to file
Path schemaFile = loader.extractSchema("messages.json", Paths.get("./"));
```

## Naming Conventions

### Snake Case → Camel Case

Java generation automatically converts Rust's `snake_case` to Java's `camelCase`:

```rust
pub struct Example {
    pub user_name: String,      // → userName in Java
    pub email_address: String,  // → emailAddress in Java
}
```

The original snake_case names are preserved via `@SerializedName` annotations for JSON compatibility.

### Serde Rename

The `#[serde(rename = "...")]` attribute is respected:

```rust
pub struct Example {
    #[serde(rename = "user")]
    pub user_name: String,  // → JSON: "user", Java field: userName
}
```

## Documentation Preservation

Doc comments (`///`) are preserved in generated code:

- **JSON Schema**: Becomes `description` field
- **Java**: Becomes JavaDoc comments

```rust
/// A greeting request.
#[derive(Serialize, Deserialize)]
pub struct GreetingRequest {
    /// The name to greet.
    pub name: String,
}
```

Generates:

```java
/**
 * A greeting request.
 */
public class GreetingRequest {
    /**
     * The name to greet.
     */
    public String name;
}
```

## Architecture

### Two-Stage Pipeline

Code generation uses a two-stage pipeline:

1. **Parse → IR**: Parse Rust source into an intermediate representation (IR)
2. **IR → Target**: Generate target code from IR

```
Rust Source
    ↓
 [Parser]
    ↓
   IR (MessageType)
    ↓
 ├─→ [JSON Schema Generator] → schema.json
 └─→ [Java Generator] → *.java
```

This architecture allows:
- Sharing parsing logic across generators
- Adding new languages without re-parsing
- Consistent transformations across all targets

### Intermediate Representation

The IR is defined in `crates/rustbridge-cli/src/codegen/ir.rs`:

```rust
pub struct MessageType {
    pub name: String,
    pub docs: Vec<String>,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: String,
    pub ty: FieldType,
    pub docs: Vec<String>,
    pub optional: bool,
    pub serde_rename: Option<String>,
}

pub enum FieldType {
    String,
    Bool,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Vec(Box<FieldType>),
    Option(Box<FieldType>),
    Custom(String),
}
```

## Limitations

### Not Supported

The following Rust features are not currently supported:

- **Enums**: Only structs are supported
- **Tuple structs**: Use named fields instead
- **Generics**: Generic structs are not supported
- **Lifetimes**: All types must be `'static`
- **Array types**: `[T; N]` not supported, use `Vec<T>`
- **Complex types**: `HashMap`, `HashSet`, etc. not supported

### Workarounds

**For enums**, use a tagged union pattern:

```rust
// Instead of:
pub enum Status {
    Success,
    Error(String),
}

// Use:
pub struct Status {
    pub status_type: String,  // "success" or "error"
    pub error_message: Option<String>,
}
```

**For HashMaps**, use a Vec of key-value pairs:

```rust
// Instead of:
pub config: HashMap<String, String>,

// Use:
pub config: Vec<ConfigEntry>,

#[derive(Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}
```

## Testing

The code generation system has comprehensive test coverage:

- **Parser tests**: Verify IR extraction from Rust source
- **Generator tests**: Verify correct output for each language
- **Integration tests**: End-to-end CLI command tests
- **Edge case tests**: Optional fields, nested types, documentation

Run tests:

```bash
cargo test -p rustbridge-cli
```

## Future Enhancements

Planned improvements:

- **Kotlin support**: Generate Kotlin data classes
- **C# support**: Generate C# records/classes
- **TypeScript support**: Generate TypeScript interfaces
- **Enum support**: Handle Rust enums with proper discriminators
- **Generic support**: Limited support for simple generics
- **Validation**: Generate validation code from doc comments
- **Builder pattern**: Optional builder class generation

## Related Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Overall system design
- [BINARY_TRANSPORT.md](./BINARY_TRANSPORT.md) - C header generation for binary transport
- [TESTING.md](./TESTING.md) - Testing conventions and guidelines
