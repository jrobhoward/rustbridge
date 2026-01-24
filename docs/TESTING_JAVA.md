# rustbridge Java Testing Conventions

This document describes the testing conventions for Java code in the rustbridge workspace (primarily `rustbridge-java/rustbridge-jni`).

## Code Quality Requirements

### Checkstyle

All Java code must pass Checkstyle validation:

```bash
cd rustbridge-java
./gradlew checkstyleMain checkstyleTest
```

This must run with **zero violations** before code is considered ready for review.

### Use of Assertions in Tests

Unlike production code, tests **may** use assert statements and unchecked casts freely:

```java
@Test
public void pluginConfig___fromValidJson___parsesCorrectly() throws IOException {
    String json = "{\"logLevel\": \"debug\"}";

    PluginConfig config = PluginConfig.fromJson(json);

    assertEquals(config.getLogLevel(), LogLevel.DEBUG);
}
```

**Rationale**: Test failures should be visible and obvious. Assertions in tests cause clear failures with backtraces, making debugging easier.

## File Organization

### Test File Structure

Tests are located alongside source files in the `test` directory, mirroring the source structure:

```
rustbridge-java/rustbridge-jni/src/
├── main/
│   └── java/
│       └── com/rustbridge/jni/
│           ├── Plugin.java
│           ├── JniPlugin.java
│           └── ...
└── test/
    └── java/
        └── com/rustbridge/jni/
            ├── PluginTest.java
            ├── JniPluginTest.java
            └── ...
```

### Test Class Naming

Test classes follow the pattern: `{ClassName}Test.java`

- `Plugin.java` → `PluginTest.java`
- `JniPlugin.java` → `JniPluginTest.java`

## Test Naming Convention

Tests follow a structured naming pattern with **triple underscores** as separators:

```
subjectUnderTest___condition___expectedResult
```

### Components

1. **subjectUnderTest**: The method or component being tested (camelCase)
2. **condition**: The specific scenario or input condition (camelCase)
3. **expectedResult**: What should happen (camelCase)

### Examples

```java
@Test
public void pluginConfig___fromEmptyJson___returnsDefaults() { ... }

@Test
public void lifecycleState___activeToStopping___transitionSucceeds() { ... }

@Test
public void ffiBuffer___fromByteArray___preservesData() { ... }

@Test
public void plugin___handleUnknownType___returnsError() throws Exception {
    // ...
}
```

### Guidelines

- Use camelCase for all parts
- Use triple underscores (`___`) only as separators between components
- Be specific but concise
- The test name should read as a specification
- Method names must be valid Java identifiers (letters, digits, underscores)

## Test Body Structure

Tests follow the **Arrange-Act-Assert** pattern, separated by blank lines (no comments):

```java
@Test
public void lifecycleState___installedToStarting___transitionSucceeds() {
    LifecycleState state = LifecycleState.INSTALLED;

    boolean canTransition = state.canTransitionTo(LifecycleState.STARTING);

    assertTrue(canTransition);
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

## Exception Testing

Test expected exceptions using `assertThrows`:

```java
@Test
public void plugin___handleInvalidMessage___throwsException() {
    Plugin plugin = new TestPlugin();
    byte[] invalidPayload = new byte[]{};

    Exception exception = assertThrows(InvalidMessageException.class, () -> {
        plugin.handleRequest("unknown", invalidPayload);
    });

    assertEquals("Unknown type: unknown", exception.getMessage());
}
```

## Test Fixtures and Setup

Use `@BeforeEach` and `@AfterEach` for test setup/teardown:

```java
public class JniPluginTest {
    private JniPlugin plugin;
    private PluginConfig config;

    @BeforeEach
    public void setUp() {
        config = new PluginConfig.Builder()
            .logLevel(LogLevel.DEBUG)
            .build();
        plugin = new JniPlugin();
    }

    @AfterEach
    public void tearDown() {
        if (plugin != null) {
            plugin.close();
        }
    }

    @Test
    public void plugin___initialize___succeeds() throws Exception {
        plugin.initialize(config);

        assertTrue(plugin.isInitialized());
    }
}
```

For class-level setup that runs once, use `@BeforeAll` and `@AfterAll`:

```java
@BeforeAll
static void setUpClass() {
    // One-time setup for all tests in this class
}

@AfterAll
static void tearDownClass() {
    // One-time cleanup for all tests in this class
}
```

## Parameterized Tests

Use parameterized tests for testing multiple scenarios:

```java
@ParameterizedTest
@ValueSource(strings = {"debug", "info", "warn", "error"})
public void logLevel___anyValidLevel___parsesCorrectly(String levelStr) {
    LogLevel level = LogLevel.parse(levelStr);

    assertNotNull(level);
}

@ParameterizedTest
@CsvSource({
    "1, ONE",
    "2, TWO",
    "3, THREE"
})
public void number___validInput___convertsCorrectly(int input, String expected) {
    String result = convertNumber(input);

    assertEquals(expected, result);
}
```

Add to build.gradle:

```gradle
testImplementation 'org.junit.jupiter:junit-jupiter-params:5.9.3'
```

## Test Assertions

Prefer specific assertions over generic ones:

```java
// Good - specific
assertEquals(expected, result);
assertTrue(result instanceof LifecycleState.ACTIVE);
assertEquals(6, error.getErrorCode());

// Avoid - too generic
assertNotNull(result);  // Only use when the actual value doesn't matter
assertTrue(!result.isEmpty());  // Use assertFalse instead
```

Use JUnit's assertion methods with descriptive messages:

```java
assertEquals(expected, actual, "Optional failure message");
assertNotEquals(unexpected, actual, "Optional failure message");
assertTrue(condition, "Optional failure message");
assertFalse(condition, "Optional failure message");
assertThrows(ExceptionType.class, () -> { /* code */ });
assertNull(value);
assertNotNull(value);
```

## Test Utilities

Create test utilities in a `test/common` package when needed:

```java
// src/test/java/com/rustbridge/test/TestUtils.java

public class TestUtils {
    public static Plugin createTestPlugin() {
        return new TestPlugin();
    }

    public static PluginContext createTestContext() {
        return new PluginContext(new PluginConfig());
    }
}

public class TestPlugin implements Plugin {
    @Override
    public void onStart(PluginContext ctx) throws Exception {
        // Test implementation
    }

    @Override
    public byte[] handleRequest(PluginContext ctx, String typeTag, byte[] payload) throws Exception {
        switch (typeTag) {
            case "echo":
                return payload;
            default:
                throw new UnknownMessageTypeException(typeTag);
        }
    }

    @Override
    public void onStop(PluginContext ctx) throws Exception {
        // Cleanup
    }
}
```

## Resource Management Tests

Use try-with-resources for resource management in tests:

```java
@Test
public void plugin___withResource___closesCorrectly() throws Exception {
    try (TestPlugin plugin = new TestPlugin()) {
        plugin.initialize();

        assertTrue(plugin.isInitialized());
    }

    assertFalse(plugin.isOpen());
}
```

## Mock Objects

Use Mockito for mocking dependencies:

```java
import static org.mockito.Mockito.*;

@Test
public void plugin___callWithMockedCallback___returnsCorrectly() throws Exception {
    PluginCallback mockCallback = mock(PluginCallback.class);
    JniPlugin plugin = new JniPlugin(mockCallback);

    plugin.handleRequest("test", new byte[]{1, 2, 3});

    verify(mockCallback).onSuccess(any());
}

@Test
public void plugin___errorInCallback___isHandled() throws Exception {
    PluginCallback mockCallback = mock(PluginCallback.class);
    doThrow(new IOException("Test error")).when(mockCallback).onSuccess(any());

    JniPlugin plugin = new JniPlugin(mockCallback);
    assertThrows(IOException.class, () -> {
        plugin.handleRequest("test", new byte[]{1, 2, 3});
    });
}
```

Add to build.gradle:

```gradle
testImplementation 'org.mockito:mockito-core:5.3.1'
testImplementation 'org.mockito:mockito-inline:5.3.1'
```

## Integration Tests

Integration tests live in `src/integrationTest/java` and test end-to-end flows:

```
rustbridge-java/rustbridge-jni/src/
└── integrationTest/
    └── java/
        └── com/rustbridge/jni/
            ├── PluginLifecycleIntegrationTest.java
            └── JniRoundtripIntegrationTest.java
```

Integration tests follow the **same naming conventions** as unit tests:

```java
@Test
public void plugin___fullLifecycle___startCallStopSucceeds() throws Exception {
    PluginConfig config = new PluginConfig.Builder()
        .logLevel(LogLevel.DEBUG)
        .build();
    JniPlugin plugin = new JniPlugin();

    plugin.initialize(config);
    plugin.onStart(createTestContext());

    byte[] response = plugin.handleRequest("echo", new byte[]{1, 2, 3});

    plugin.onStop(createTestContext());

    assertArrayEquals(new byte[]{1, 2, 3}, response);
}
```

## Annotation Reference

### Test Methods

```java
@Test                          // Mark a test method
@DisplayName("Custom name")    // Provide a readable test name
@Tag("performance")            // Tag for test categorization
@Disabled                      // Disable a test temporarily
@Timeout(5, TimeUnit.SECONDS)  // Set test timeout
```

### Parameterization

```java
@ParameterizedTest
@ValueSource(ints = {1, 2, 3})
@CsvSource({...})
@MethodSource("methodName")
```

## Running Tests

```bash
# Run all tests
cd rustbridge-java
./gradlew test

# Run tests for a specific subproject
./gradlew :rustbridge-jni:test

# Run a specific test class
./gradlew test --tests "com.rustbridge.jni.PluginTest"

# Run a specific test method
./gradlew test --tests "com.rustbridge.jni.PluginTest.pluginConfig___fromEmptyJson___returnsDefaults"

# Run with output
./gradlew test --info

# Run with coverage
./gradlew test jacocoTestReport

# Run integration tests
./gradlew integrationTest
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
open rustbridge-jni/build/reports/jacoco/test/html/index.html
```

## Dependencies

Add to `build.gradle` for Java test setup:

```gradle
dependencies {
    // Testing
    testImplementation 'org.junit.jupiter:junit-jupiter:5.9.3'
    testImplementation 'org.junit.jupiter:junit-jupiter-params:5.9.3'
    testImplementation 'org.mockito:mockito-core:5.3.1'
    testImplementation 'org.mockito:mockito-inline:5.3.1'

    // Code quality
    checkstyle 'com.puppycrawl.tools:checkstyle:10.12.3'
}

test {
    useJUnitPlatform()
}
```

## Best Practices

1. **Test names as specifications**: A test name should read like a sentence describing behavior
2. **Single responsibility**: Each test should verify one logical outcome
3. **No side effects**: Tests should be independent and idempotent
4. **Minimal setup**: Only create what's needed for the test
5. **Clear assertions**: Use descriptive assertion messages for failures
6. **Avoid test interdependencies**: Don't rely on other tests running first
7. **Fail fast**: Stop execution as soon as a condition is unmet
8. **Test behavior, not implementation**: Focus on what the code does, not how it does it
