# rustbridge Kotlin Testing Conventions

This document describes the testing conventions for Kotlin code in the rustbridge workspace (primarily `rustbridge-java/rustbridge-ffm`).

> **Note**: This guide follows the [shared testing conventions](./TESTING.md#cross-language-testing-conventions) used across all rustbridge languages, including the triple-underscore naming pattern and Arrange-Act-Assert structure.

## Code Quality Requirements

### Ktlint

All Kotlin code must pass ktlint formatting:

```bash
cd rustbridge-java
./gradlew ktlintCheck
```

This must run with **zero violations** before code is considered ready for review.

### Use of `.let`, `.run`, and Error Handling in Tests

Unlike production code, tests **may** use `.let`, `.run`, and non-null assertions (`!!`) freely:

```kotlin
@Test
fun pluginConfig___fromValidJson___parsesCorrectly() {
    val json = """{"logLevel": "debug"}"""

    val config = PluginConfig.fromJson(json)!!

    assertEquals(config.logLevel, LogLevel.DEBUG)
}
```

**Rationale**: Test failures should be visible and obvious. Using `!!` in tests causes a clear exception with a backtrace, making debugging easier.

## File Organization

### Test File Structure

Tests are located alongside source files in the `test` directory, mirroring the source structure:

```
rustbridge-java/rustbridge-ffm/src/
├── main/
│   └── java/
│       └── com/rustbridge/ffm/
│           ├── Plugin.kt
│           ├── FfmPlugin.kt
│           └── ...
└── test/
    └── java/
        └── com/rustbridge/ffm/
            ├── PluginTest.kt
            ├── FfmPluginTest.kt
            └── ...
```

### Test Class Naming

Test classes follow the pattern: `{ClassName}Test.kt`

- `Plugin.kt` → `PluginTest.kt`
- `FfmPlugin.kt` → `FfmPluginTest.kt`

## Test Naming Convention

Tests follow a structured naming pattern with **triple underscores** as separators:

```
subjectUnderTest___condition___expectedResult
```

### Components

1. **subjectUnderTest**: The method, function, or component being tested (camelCase)
2. **condition**: The specific scenario or input condition (camelCase)
3. **expectedResult**: What should happen (camelCase)

### Examples

```kotlin
@Test
fun pluginConfig___fromEmptyJson___returnsDefaults() { ... }

@Test
fun lifecycleState___activeToStopping___transitionSucceeds() { ... }

@Test
fun ffiBuffer___fromByteArray___preservesData() { ... }

@Test
fun plugin___handleUnknownType___returnsError() = runTest {
    // ...
}
```

### Guidelines

- Use camelCase for all parts
- Use triple underscores (`___`) only as separators between components
- Be specific but concise
- The test name should read as a specification
- Use backticks in the function name to escape special characters if needed: `` `subject___condition___result`() ``

## Test Body Structure

Tests follow the **Arrange-Act-Assert** pattern, separated by blank lines (no comments):

```kotlin
@Test
fun lifecycleState___installedToStarting___transitionSucceeds() {
    val state = LifecycleState.INSTALLED

    val canTransition = state.canTransitionTo(LifecycleState.STARTING)

    assertTrue(canTransition)
}
```

### Structure

1. **Arrange**: Set up test data and preconditions (first block)
2. **Act**: Execute the code under test (second block)
3. **Assert**: Verify the results (third block)

### Guidelines

- Separate sections with a single blank line
- Do NOT add `// Arrange`, `// Act`, `// Assert` comments
- Keep each section focused and minimal
- For simple tests, sections may be combined if clarity is maintained

## Async/Coroutine Tests

For coroutine tests, use `runTest` from `kotlinx-coroutines-test`:

```kotlin
@Test
fun plugin___onStart___transitionsToActive() = runTest {
    val plugin = TestPlugin()
    val ctx = PluginContext(PluginConfig())

    val result = plugin.onStart(ctx)

    assertTrue(result.isSuccess)
}
```

Add to build.gradle:

```gradle
testImplementation 'org.jetbrains.kotlinx:kotlinx-coroutines-test:1.7.3'
```

## Test Assertions

Prefer specific assertions over generic ones:

```kotlin
// Good - specific
assertEquals(result, expectedValue)
assertTrue(result is LifecycleState.ACTIVE)
assertEquals(error.errorCode, 6)

// Avoid - too generic
assertTrue(result != null)  // Only use when the actual value doesn't matter
```

Use Kotlin's assertion functions:

```kotlin
assertEquals(expected, actual, "Optional message")
assertNotEquals(unexpected, actual)
assertTrue(condition, "Optional message")
assertFalse(condition, "Optional message")
assertThrows<ExceptionType> { /* code */ }
```

## Test Fixtures and Setup

Use `@BeforeEach` and `@AfterEach` for test setup/teardown:

```kotlin
class FfmPluginTest {
    private lateinit var plugin: FfmPlugin
    private lateinit var config: PluginConfig

    @BeforeEach
    fun setUp() {
        config = PluginConfig(logLevel = LogLevel.DEBUG)
        plugin = FfmPlugin()
    }

    @AfterEach
    fun tearDown() {
        // Cleanup if needed
    }

    @Test
    fun plugin___initialize___succeeds() {
        plugin.initialize(config)

        assertTrue(plugin.isInitialized)
    }
}
```

## Parameterized Tests

Use parameterized tests for testing multiple scenarios:

```kotlin
@ParameterizedTest
@ValueSource(strings = ["debug", "info", "warn", "error"])
fun logLevel___anyValidLevel___parsesCorrectly(levelStr: String) {
    val level = LogLevel.parse(levelStr)

    assertNotNull(level)
}

@ParameterizedTest
@CsvSource(
    "1, ONE",
    "2, TWO",
    "3, THREE"
)
fun number___validInput___convertsCorrectly(input: Int, expected: String) {
    val result = convertNumber(input)

    assertEquals(expected, result)
}
```

Add to build.gradle:

```gradle
testImplementation 'org.junit.jupiter:junit-jupiter-params:5.9.3'
```

## Test Utilities

Create test utilities in a `test/common` module when needed:

```kotlin
// src/test/java/com/rustbridge/test/TestUtils.kt

object TestUtils {
    fun createTestPlugin(): Plugin {
        return TestPlugin()
    }

    fun createTestContext(): PluginContext {
        return PluginContext(PluginConfig())
    }
}

class TestPlugin : Plugin {
    override suspend fun onStart(ctx: PluginContext) {
        // Test implementation
    }

    override suspend fun handleRequest(
        ctx: PluginContext,
        typeTag: String,
        payload: ByteArray
    ): ByteArray {
        return when (typeTag) {
            "echo" -> payload
            else -> throw UnknownMessageTypeException(typeTag)
        }
    }

    override suspend fun onStop(ctx: PluginContext) {
        // Cleanup
    }
}
```

## Resource Management Tests

Use `use` for resource management in tests:

```kotlin
@Test
fun plugin___withResource___closesCorrectly() {
    TestPlugin().use { plugin ->
        plugin.initialize()

        assertTrue(plugin.isInitialized)
    }

    assertFalse(plugin.isOpen)
}
```

## Mock Objects

Use Mockito for mocking dependencies:

```kotlin
import org.mockito.kotlin.*

@Test
fun plugin___callWithMockedCallback___returnsCorrectly() {
    val mockCallback = mock<PluginCallback>()
    val plugin = FfmPlugin(callback = mockCallback)

    plugin.handleRequest("test", byteArrayOf(1, 2, 3))

    verify(mockCallback).onSuccess(any())
}
```

Add to build.gradle:

```gradle
testImplementation 'org.mockito.kotlin:mockito-kotlin:5.1.0'
testImplementation 'org.mockito:mockito-core:5.3.1'
```

## Integration Tests

Integration tests live in `src/integrationTest/java` and test end-to-end flows:

```
rustbridge-java/rustbridge-ffm/src/
└── integrationTest/
    └── java/
        └── com/rustbridge/ffm/
            ├── PluginLifecycleIntegrationTest.kt
            └── FfmRoundtripIntegrationTest.kt
```

Integration tests follow the **same naming conventions** as unit tests:

```kotlin
@Test
fun plugin___fullLifecycle___startCallStopSucceeds() {
    val config = PluginConfig(logLevel = LogLevel.DEBUG)
    val plugin = FfmPlugin()

    plugin.initialize(config)
    plugin.onStart(createTestContext())

    val response = plugin.handleRequest("echo", byteArrayOf(1, 2, 3))

    plugin.onStop(createTestContext())

    assertContentEquals(byteArrayOf(1, 2, 3), response)
}
```

## Running Tests

```bash
# Run all tests
cd rustbridge-java
./gradlew test

# Run tests for a specific subproject
./gradlew :rustbridge-ffm:test

# Run a specific test class
./gradlew test --tests "com.rustbridge.ffm.PluginTest"

# Run a specific test method
./gradlew test --tests "com.rustbridge.ffm.PluginTest.pluginConfig___fromEmptyJson___returnsDefaults"

# Run with output
./gradlew test --info

# Run with coverage
./gradlew test jacocoTestReport
```

## Test Coverage

Measure test coverage using Jacoco:

```bash
cd rustbridge-java
./gradlew test jacocoTestReport
```

Coverage goals:
- Core logic: >85%
- FFI boundary: >80%
- Error handling: >80%
- Overall: >70%

View the HTML report:

```bash
open rustbridge-ffm/build/reports/jacoco/test/html/index.html
```

## Dependencies

Add to `build.gradle` for Kotlin test setup:

```gradle
dependencies {
    // Testing
    testImplementation 'org.junit.jupiter:junit-jupiter:5.9.3'
    testImplementation 'org.junit.jupiter:junit-jupiter-params:5.9.3'
    testImplementation 'org.jetbrains.kotlinx:kotlinx-coroutines-test:1.7.3'
    testImplementation 'org.mockito.kotlin:mockito-kotlin:5.1.0'
    testImplementation 'org.mockito:mockito-core:5.3.1'

    // Code quality
    ktlintClasspath 'com.pinterest:ktlint:1.1.0'
}
```

## Test Timeouts

**Always add timeouts to integration tests** to prevent builds from hanging indefinitely. This is especially important for tests involving:
- Native library loading (FFI)
- Concurrent operations
- Resource cleanup / lifecycle management

### Per-Test Timeout

```kotlin
import org.junit.jupiter.api.Timeout
import java.util.concurrent.TimeUnit

@Test
@Timeout(value = 30, unit = TimeUnit.SECONDS)
fun plugin___fullLifecycle___completesSuccessfully() {
    // Test that must complete within 30 seconds
}
```

### Class-Level Timeout

Apply to all tests in a class:

```kotlin
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class PluginIntegrationTest {
    @Test
    fun testOne() { ... }  // 60s timeout

    @Test
    fun testTwo() { ... }  // 60s timeout
}
```

### Coroutine Timeouts

For coroutine tests, use `withTimeout`:

```kotlin
@Test
fun plugin___asyncOperation___completesInTime() = runTest {
    withTimeout(30.seconds) {
        val result = plugin.asyncCall("test", request)
        assertNotNull(result)
    }
}
```

### Global Timeout via Gradle

Configure default timeout in `build.gradle.kts`:

```kotlin
tasks.withType<Test> {
    useJUnitPlatform()
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}
```

### Timeout Guidelines

| Test Type | Recommended Timeout |
|-----------|-------------------|
| Unit tests | 5-10 seconds |
| Integration tests | 30-60 seconds |
| Performance/stress tests | 2-5 minutes |

**Rationale**: Tests that hang indefinitely can block CI pipelines and developer workflows. Timeouts provide fail-fast behavior and clear error messages about which tests are problematic.

## Test Isolation

For tests that modify global state or interact with native libraries, isolation prevents interference between tests.

### Process Isolation

Fork a new JVM for each test class:

```kotlin
// build.gradle.kts
tasks.withType<Test> {
    forkEvery = 1  // New JVM per test class
}
```

### JUnit @Isolated Annotation (JUnit 5.9+)

Ensure a test runs alone, not concurrently with others:

```kotlin
import org.junit.jupiter.api.parallel.Isolated

@Isolated  // Runs without other tests in parallel
class GlobalStateTest {
    @Test
    fun test___modifiesGlobalState___succeeds() { ... }
}
```

### Disable Parallel Execution

For test classes that share state:

```kotlin
import org.junit.jupiter.api.parallel.Execution
import org.junit.jupiter.api.parallel.ExecutionMode

@Execution(ExecutionMode.SAME_THREAD)
class SequentialTest {
    // All tests run sequentially in the same thread
}
```

### When to Use Isolation

- Tests that load/unload native libraries
- Tests that modify global configuration
- Tests with shared callback handlers
- Tests involving plugin reload cycles

## Best Practices

1. **Test names as specifications**: A test name should read like a sentence describing behavior
2. **Single responsibility**: Each test should verify one logical outcome
3. **No side effects**: Tests should be independent and idempotent
4. **Minimal setup**: Only create what's needed for the test
5. **Clear assertions**: Use descriptive assertion messages for failures
6. **Avoid test interdependencies**: Don't rely on other tests running first
7. **Always use timeouts**: Integration tests must have explicit timeouts to prevent hangs
