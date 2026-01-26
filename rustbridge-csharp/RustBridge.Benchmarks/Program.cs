using BenchmarkDotNet.Configs;
using BenchmarkDotNet.Running;
using RustBridge.Benchmarks;

// Check if hello-plugin is available before running benchmarks
var pluginPath = BenchmarkHelper.FindHelloPlugin();
if (pluginPath == null)
{
    Console.Error.WriteLine("Error: hello-plugin not found.");
    Console.Error.WriteLine("Please build it first: cargo build --release -p hello-plugin");
    return 1;
}

Console.WriteLine($"Using plugin: {pluginPath}");
Console.WriteLine();

// Run benchmarks based on command line args
var config = DefaultConfig.Instance;

#if DEBUG
Console.WriteLine("Warning: Running in DEBUG mode. Use Release for accurate benchmarks:");
Console.WriteLine("  dotnet run -c Release");
Console.WriteLine();
config = new DebugInProcessConfig();
#endif

var summary = BenchmarkSwitcher
    .FromAssembly(typeof(Program).Assembly)
    .Run(args, config);

return 0;
