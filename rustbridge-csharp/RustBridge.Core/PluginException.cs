namespace RustBridge;

/// <summary>
/// Exception thrown by RustBridge plugin operations.
/// </summary>
public class PluginException : Exception
{
    /// <summary>
    /// Get the error code.
    /// </summary>
    public int ErrorCode { get; }

    /// <summary>
    /// Create a new PluginException.
    /// </summary>
    /// <param name="message">The error message.</param>
    public PluginException(string message) : base(message)
    {
        ErrorCode = 0;
    }

    /// <summary>
    /// Create a new PluginException with an error code.
    /// </summary>
    /// <param name="errorCode">The error code.</param>
    /// <param name="message">The error message.</param>
    public PluginException(int errorCode, string message) : base(message)
    {
        ErrorCode = errorCode;
    }

    /// <summary>
    /// Create a new PluginException with a cause.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The underlying cause.</param>
    public PluginException(string message, Exception? innerException) : base(message, innerException)
    {
        ErrorCode = 0;
    }

    /// <summary>
    /// Create a new PluginException with an error code and cause.
    /// </summary>
    /// <param name="errorCode">The error code.</param>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The underlying cause.</param>
    public PluginException(int errorCode, string message, Exception? innerException) : base(message, innerException)
    {
        ErrorCode = errorCode;
    }
}
