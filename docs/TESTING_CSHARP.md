# rustbridge C# Testing Conventions

This document describes the testing conventions for C# code in the rustbridge workspace (`rustbridge-csharp/`).

## Code Quality Requirements

### Build with Warnings as Errors

All C# code must build without warnings:

```bash
cd rustbridge-csharp
dotnet build
```

This is enforced via `Directory.Build.props`:

```xml
<TreatWarningsAsErrors>true</TreatWarningsAsErrors>
```

## File Organization

### Project Structure

```
rustbridge-csharp/
├── RustBridge.sln
├── Directory.Build.props           # Shared build settings
├── RustBridge.Core/                # Core abstractions
│   ├── IPlugin.cs
│   ├── PluginConfig.cs
│   ├── PluginException.cs
│   ├── LifecycleState.cs
│   ├── LogLevel.cs
│   ├── BundleLoader.cs
│   └── MinisignVerifier.cs
├── RustBridge.Native/              # P/Invoke bindings
│   ├── NativeBindings.cs
│   ├── NativePlugin.cs
│   └── NativePluginLoader.cs
└── RustBridge.Tests/               # All tests
    ├── LifecycleStateTests.cs
    ├── PluginConfigTests.cs
    ├── HelloPluginIntegrationTest.cs
    └── BinaryTransportTest.cs
```

### Test Class Naming

Test classes follow the pattern: `{ClassName}Tests.cs` or `{Feature}Test.cs`

- `LifecycleState.cs` → `LifecycleStateTests.cs`
- `PluginConfig.cs` → `PluginConfigTests.cs`
- Integration tests → `HelloPluginIntegrationTest.cs`

## Test Naming Convention

Tests follow a structured naming pattern with **triple underscores** as separators:

```
SubjectUnderTest___Condition___ExpectedResult
```

### Components

1. **SubjectUnderTest**: The method or component being tested (PascalCase)
2. **Condition**: The specific scenario or input condition (PascalCase)
3. **ExpectedResult**: What should happen (PascalCase)

### Examples

```csharp
[Fact]
public void FromCode___ValidCode___ReturnsCorrectState() { ... }

[Fact]
public void Call___EchoMessage___ReturnsMessageWithLength() { ... }

[Fact]
public void Dispose___AfterDispose___StateIsStopped() { ... }

[Fact]
public void Call___InvalidTypeTag___ThrowsWithErrorCode6() { ... }
```

### Guidelines

- Use PascalCase for all parts
- Use triple underscores (`___`) only as separators between components
- Be specific but concise
- The test name should read as a specification

## Test Body Structure

Tests follow the **Arrange-Act-Assert** pattern, separated by blank lines (no comments):

```csharp
[Fact]
public void FromCode___ValidCode___ReturnsCorrectState()
{
    int code = 2;

    var result = LifecycleStateExtensions.FromCode(code);

    Assert.Equal(LifecycleState.Active, result);
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

Test expected exceptions using `Assert.Throws`:

```csharp
[Fact]
public void FromCode___InvalidCode___ThrowsArgumentException()
{
    int invalidCode = 99;

    Assert.Throws<ArgumentException>(() =>
        LifecycleStateExtensions.FromCode(invalidCode));
}

[Fact]
public void Call___InvalidTypeTag___ThrowsWithErrorCode6()
{
    var ex = Assert.Throws<PluginException>(() =>
        _plugin.Call("invalid.type.tag", """{"test": true}"""));

    Assert.Equal(6, ex.ErrorCode);
}
```

## Test Fixtures and Setup

Use constructor and `IDisposable` for test setup/teardown:

```csharp
public class HelloPluginIntegrationTest : IDisposable
{
    private readonly IPlugin? _plugin;
    private readonly string? _skipReason;

    public HelloPluginIntegrationTest()
    {
        var libraryPath = FindHelloPlugin();
        if (libraryPath == null)
        {
            _skipReason = "hello-plugin not found";
            return;
        }

        _plugin = NativePluginLoader.Load(libraryPath);
    }

    public void Dispose()
    {
        _plugin?.Dispose();
    }

    [SkippableFact]
    public void Load___ValidPlugin___StateIsActive()
    {
        Skip.If(_skipReason != null, _skipReason);

        Assert.Equal(LifecycleState.Active, _plugin!.State);
    }
}
```

For shared context across multiple test classes, use `IClassFixture<T>`:

```csharp
public class PluginFixture : IDisposable
{
    public IPlugin Plugin { get; }

    public PluginFixture()
    {
        Plugin = NativePluginLoader.Load("path/to/plugin.dll");
    }

    public void Dispose() => Plugin.Dispose();
}

public class PluginTests : IClassFixture<PluginFixture>
{
    private readonly IPlugin _plugin;

    public PluginTests(PluginFixture fixture)
    {
        _plugin = fixture.Plugin;
    }
}
```

## Parameterized Tests

Use `[Theory]` with data attributes for testing multiple scenarios:

```csharp
[Theory]
[InlineData(0, LifecycleState.Installed)]
[InlineData(1, LifecycleState.Starting)]
[InlineData(2, LifecycleState.Active)]
[InlineData(3, LifecycleState.Stopping)]
[InlineData(4, LifecycleState.Stopped)]
[InlineData(5, LifecycleState.Failed)]
public void FromCode___ValidCode___ReturnsCorrectState(int code, LifecycleState expected)
{
    var result = LifecycleStateExtensions.FromCode(code);

    Assert.Equal(expected, result);
}

[Theory]
[InlineData(-1)]
[InlineData(6)]
[InlineData(100)]
public void FromCode___InvalidCode___ThrowsArgumentException(int code)
{
    Assert.Throws<ArgumentException>(() =>
        LifecycleStateExtensions.FromCode(code));
}
```

## Test Assertions

Prefer specific assertions over generic ones:

```csharp
// Good - specific
Assert.Equal(expected, result);
Assert.True(state.CanHandleRequests());
Assert.Equal(6, error.ErrorCode);
Assert.StartsWith("user-", result.UserId);
Assert.Contains("Alice", result.Greeting);

// Avoid - too generic
Assert.NotNull(result);  // Only use when the actual value doesn't matter
Assert.True(!result.IsEmpty);  // Use Assert.False instead
```

Common assertions:

```csharp
Assert.Equal(expected, actual);
Assert.NotEqual(unexpected, actual);
Assert.True(condition);
Assert.False(condition);
Assert.Throws<ExceptionType>(() => { /* code */ });
Assert.Null(value);
Assert.NotNull(value);
Assert.Empty(collection);
Assert.Contains(item, collection);
Assert.StartsWith(prefix, value);
Assert.EndsWith(suffix, value);
```

## Skippable Tests

For tests that depend on external resources (like native plugins), use `SkippableFact`:

```csharp
// Add package: Xunit.SkippableFact

[SkippableFact]
public void Call___EchoMessage___ReturnsMessageWithLength()
{
    Skip.If(_plugin == null, "Plugin not available");

    var response = _plugin.Call("echo", """{"message": "Hello"}""");

    Assert.Contains("Hello", response);
}
```

## Async Tests

For async tests, use `async Task` return type:

```csharp
[SkippableFact]
public async Task Call___ConcurrentCalls___AllSucceed()
{
    Skip.If(_plugin == null, "Plugin not available");

    const int concurrentCalls = 100;
    var tasks = new Task<string>[concurrentCalls];

    for (int i = 0; i < concurrentCalls; i++)
    {
        var message = $"Message {i}";
        tasks[i] = Task.Run(() =>
            _plugin.Call("echo", $$$"""{"message": "{{{message}}}"}"""));
    }

    var results = await Task.WhenAll(tasks);

    Assert.All(results, r => Assert.NotNull(r));
}
```

## Test Categories

Use `[Trait]` to categorize tests:

```csharp
[Trait("Category", "Integration")]
public class HelloPluginIntegrationTest { ... }

[Trait("Category", "Unit")]
public class LifecycleStateTests { ... }
```

Run specific categories:

```bash
# Run only integration tests
dotnet test --filter "Category=Integration"

# Run only unit tests
dotnet test --filter "Category=Unit"
```

## Resource Management Tests

Use `using` statements for resource management:

```csharp
[SkippableFact]
public void Dispose___AfterDispose___StateIsStopped()
{
    Skip.If(_skipReason != null, _skipReason);

    var libraryPath = FindHelloPlugin()!;
    var plugin = NativePluginLoader.Load(libraryPath);

    plugin.Dispose();

    Assert.Equal(LifecycleState.Stopped, plugin.State);
}
```

## Running Tests

```bash
# Run all tests
cd rustbridge-csharp
dotnet test

# Run with detailed output
dotnet test --logger "console;verbosity=detailed"

# Run specific test class
dotnet test --filter "FullyQualifiedName~LifecycleStateTests"

# Run specific test method
dotnet test --filter "FullyQualifiedName~FromCode___ValidCode___ReturnsCorrectState"

# Run by category
dotnet test --filter "Category=Integration"

# Run with coverage (requires coverlet)
dotnet test --collect:"XPlat Code Coverage"
```

## Test Coverage

Measure test coverage using coverlet (included in test project):

```bash
dotnet test --collect:"XPlat Code Coverage"
```

Coverage goals:
- Core logic: >85%
- FFI boundary: >80%
- Error handling: >80%
- Overall: >70%

## Dependencies

The test project includes:

```xml
<ItemGroup>
  <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.8.0" />
  <PackageReference Include="xunit" Version="2.6.2" />
  <PackageReference Include="xunit.runner.visualstudio" Version="2.5.4" />
  <PackageReference Include="Xunit.SkippableFact" Version="1.4.13" />
  <PackageReference Include="coverlet.collector" Version="6.0.0" />
</ItemGroup>
```

## Integration Test Setup

Integration tests require the hello-plugin to be built:

```bash
# Build the plugin first
cargo build --release -p hello-plugin

# Then run C# tests
cd rustbridge-csharp
dotnet test
```

The integration tests automatically search for the plugin in:
- `../target/release/`
- `../target/debug/`

If the plugin isn't found, tests are skipped with a helpful message.

## Unsafe Code in Tests

The test project allows unsafe code for binary struct tests:

```xml
<AllowUnsafeBlocks>true</AllowUnsafeBlocks>
```

Use unsafe code sparingly and only for FFI-related tests:

```csharp
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public unsafe struct SmallRequestRaw : IBinaryStruct
{
    public byte Version;
    private fixed byte _reserved[3];
    private fixed byte _key[64];
    public uint KeyLen;
    public uint Flags;

    public int ByteSize => 76;
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
9. **Use SkippableFact for external dependencies**: Tests that need native plugins should skip gracefully
10. **Dispose resources properly**: Use `IDisposable` pattern for test fixtures
