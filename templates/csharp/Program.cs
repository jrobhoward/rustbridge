using System.Text.Json;
using RustBridge;
using RustBridge.Native;

// Use camelCase to match Rust's serde conventions
var jsonOptions = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
};

// TODO: Update this path to your .rbp bundle file
var bundlePath = "my-plugin-1.0.0.rbp";

using var bundleLoader = BundleLoader.Create()
    .WithBundlePath(bundlePath)
    .WithSignatureVerification(false)
    .Build();
var libraryPath = bundleLoader.ExtractLibrary();

using var plugin = NativePluginLoader.Load(libraryPath);

// Example: Call the "echo" message type
var request = new EchoRequest("Hello from C#!");
var requestJson = JsonSerializer.Serialize(request, jsonOptions);

var responseJson = plugin.Call("echo", requestJson);
var response = JsonSerializer.Deserialize<EchoResponse>(responseJson, jsonOptions);

Console.WriteLine($"Response: {response?.Message}");
Console.WriteLine($"Length: {response?.Length}");

// Type declarations must come after top-level statements
record EchoRequest(string Message);
record EchoResponse(string Message, int Length);
