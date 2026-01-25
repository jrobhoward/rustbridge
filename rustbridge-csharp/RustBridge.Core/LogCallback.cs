namespace RustBridge;

/// <summary>
/// Delegate for receiving log messages from plugins.
/// </summary>
/// <param name="level">The log level.</param>
/// <param name="target">The log target (usually module path).</param>
/// <param name="message">The log message.</param>
public delegate void LogCallback(LogLevel level, string target, string message);
