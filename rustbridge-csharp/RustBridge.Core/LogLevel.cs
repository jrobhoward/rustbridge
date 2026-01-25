namespace RustBridge;

/// <summary>
/// Log levels for plugin logging.
/// </summary>
public enum LogLevel
{
    /// <summary>
    /// Trace level - very detailed debugging information.
    /// </summary>
    Trace = 0,

    /// <summary>
    /// Debug level - debugging information.
    /// </summary>
    Debug = 1,

    /// <summary>
    /// Info level - general information.
    /// </summary>
    Info = 2,

    /// <summary>
    /// Warn level - warning messages.
    /// </summary>
    Warn = 3,

    /// <summary>
    /// Error level - error messages.
    /// </summary>
    Error = 4,

    /// <summary>
    /// Off - disable logging.
    /// </summary>
    Off = 5
}

/// <summary>
/// Extension methods for <see cref="LogLevel"/>.
/// </summary>
public static class LogLevelExtensions
{
    /// <summary>
    /// Get the level from a numeric code.
    /// </summary>
    /// <param name="code">The level code.</param>
    /// <returns>The corresponding level, or <see cref="LogLevel.Off"/> if invalid.</returns>
    public static LogLevel FromCode(int code)
    {
        if (Enum.IsDefined(typeof(LogLevel), code))
        {
            return (LogLevel)code;
        }
        return LogLevel.Off;
    }
}
