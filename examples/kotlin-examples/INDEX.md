# Kotlin Examples Index

## Quick Navigation

| Document                             | Purpose                     | Audience                 |
|--------------------------------------|-----------------------------|--------------------------|
| **[QUICKSTART.md](./QUICKSTART.md)** | Get running in 5 minutes    | New users                |
| **[README.md](./README.md)**         | Overview and features       | All users                |
| **[COMPARISON.md](./COMPARISON.md)** | Java vs Kotlin side-by-side | Java/Kotlin developers   |
| **[STRATEGY.md](./STRATEGY.md)**     | Why we chose this approach  | Contributors, architects |

## Example Code

| File                                                                                             | What It Shows                                         | Complexity      |
|--------------------------------------------------------------------------------------------------|-------------------------------------------------------|-----------------|
| **[BasicExample.kt](./src/main/kotlin/com/rustbridge/examples/BasicExample.kt)**                 | Core functionality, data classes, extension functions | ⭐ Beginner      |
| **[LoggingExample.kt](./src/main/kotlin/com/rustbridge/examples/LoggingExample.kt)**             | Custom log callbacks, colored output                  | ⭐⭐ Intermediate |
| **[ErrorHandlingExample.kt](./src/main/kotlin/com/rustbridge/examples/ErrorHandlingExample.kt)** | Sealed classes, Result type, runCatching              | ⭐⭐⭐ Advanced    |

## Learning Path

### 1. First Time Users

1. Read [QUICKSTART.md](./QUICKSTART.md)
2. Build and run [BasicExample.kt](./src/main/kotlin/com/rustbridge/examples/BasicExample.kt)
3. Skim [README.md](./README.md) for overview

### 2. Java Developers

1. Read [COMPARISON.md](./COMPARISON.md) to see differences
2. Run [BasicExample.kt](./src/main/kotlin/com/rustbridge/examples/BasicExample.kt)
3. Copy patterns you like into your Java code

### 3. Kotlin Developers

1. Skim [QUICKSTART.md](./QUICKSTART.md) for setup
2. Review all three examples
3. Use extension functions from examples in your code

### 4. Contributors/Architects

1. Read [STRATEGY.md](./STRATEGY.md) for design rationale
2. Review example implementations
3. Propose enhancements via issues/PRs

## Key Concepts Demonstrated

### Kotlin Language Features

- ✅ Data classes (concise POJOs)
- ✅ Extension functions (add methods to existing classes)
- ✅ `use` blocks (automatic resource management)
- ✅ Inline functions with reified types (type-safe generics)
- ✅ String templates (clean string formatting)
- ✅ When expressions (pattern matching)
- ✅ Sealed classes (type-safe alternatives)
- ✅ Object expressions (anonymous classes)
- ✅ Collection operators (map, filter, etc.)
- ✅ Trailing lambdas (DSL-like syntax)

### rustbridge Patterns

- ✅ Plugin lifecycle (load → use → auto-close)
- ✅ Type-safe message passing (with generics)
- ✅ Log callback integration
- ✅ Error handling (exceptions and Results)
- ✅ Configuration (PluginConfig builder)
- ✅ Resource cleanup (AutoCloseable)

## Common Use Cases

| Use Case                | Example        | File                    |
|-------------------------|----------------|-------------------------|
| Simple request/response | Echo, Greet    | BasicExample.kt         |
| Creating resources      | User creation  | BasicExample.kt         |
| Math operations         | Addition       | BasicExample.kt         |
| Custom logging          | Colored logs   | LoggingExample.kt       |
| Log counting            | Stats tracking | LoggingExample.kt       |
| Error handling          | Unknown types  | ErrorHandlingExample.kt |
| Batch operations        | Multiple calls | BasicExample.kt         |
| Type-safe errors        | Sealed classes | ErrorHandlingExample.kt |

## Building and Running

### Build Everything

```bash
# From kotlin-examples directory
./gradlew build
```

### Run All Examples

```bash
./gradlew run
```

### Run Specific Example

```bash
./gradlew runBasic
./gradlew runLogging
./gradlew runErrorHandling
```

### IDE Support

- **IntelliJ IDEA**: Open directory, auto-imports Gradle project
- **VS Code**: Install Kotlin + Gradle extensions

## Extending the Examples

### Add Your Own Message Type

1. Define data classes in your code
2. Add handler to hello-plugin (Rust side)
3. Call with `plugin.callTyped<YourResponse>("your.type", YourRequest(...))`

### Add Your Own Example

1. Create `src/main/kotlin/com/rustbridge/examples/YourExample.kt`
2. Add `fun main()` entry point
3. Add task to `build.gradle.kts`:
   ```kotlin
   tasks.register<JavaExec>("runYour") {
       classpath = sourceSets.main.get().runtimeClasspath
       mainClass.set("com.rustbridge.examples.YourExampleKt")
   }
   ```

### Create Reusable Utilities

Extract common patterns into a utility file:

```kotlin
// src/main/kotlin/com/rustbridge/examples/Utils.kt
inline fun <reified T> Plugin.callTyped(...): T { ... }
fun findPluginPath(): String { ... }
class MyLogCallback : LogCallback { ... }
```

## FAQ

**Q: Do I need to add Kotlin to my Java project?**
No! Use the Java API directly. These examples are optional.

**Q: Can I use these examples as templates?**
Yes! Copy and modify freely (MIT/Apache-2.0 license).

**Q: Should I use FFM or JNI?**
FFM (Java 21+) is preferred. JNI (Java 8+) for compatibility.

**Q: Where's the Kotlin wrapper library?**
There isn't one - Kotlin uses the Java API directly via interop.

**Q: Can I contribute more examples?**
Yes! See [STRATEGY.md](./STRATEGY.md) and submit a PR.

## Next Steps

- Try the examples
- Read the [main rustbridge documentation](../../README.md)
- Check out [ARCHITECTURE.md](../../docs/ARCHITECTURE.md)
- Build your own plugin!
