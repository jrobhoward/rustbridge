namespace RustBridge;

/// <summary>
/// Plugin lifecycle states.
/// <para>
/// The lifecycle follows this state machine:
/// <code>
/// Installed -> Starting -> Active -> Stopping -> Stopped
///                 ^                      |
///                 +----------------------+ (restart)
///             Any state -> Failed (on error)
/// </code>
/// </para>
/// </summary>
public enum LifecycleState
{
    /// <summary>
    /// Plugin is installed but not yet initialized.
    /// </summary>
    Installed = 0,

    /// <summary>
    /// Plugin is starting up.
    /// </summary>
    Starting = 1,

    /// <summary>
    /// Plugin is active and ready to handle requests.
    /// </summary>
    Active = 2,

    /// <summary>
    /// Plugin is shutting down.
    /// </summary>
    Stopping = 3,

    /// <summary>
    /// Plugin has been stopped.
    /// </summary>
    Stopped = 4,

    /// <summary>
    /// Plugin has failed.
    /// </summary>
    Failed = 5
}

/// <summary>
/// Extension methods for <see cref="LifecycleState"/>.
/// </summary>
public static class LifecycleStateExtensions
{
    /// <summary>
    /// Get the state from a numeric code.
    /// </summary>
    /// <param name="code">The state code.</param>
    /// <returns>The corresponding state.</returns>
    /// <exception cref="ArgumentException">If the code is invalid.</exception>
    public static LifecycleState FromCode(int code)
    {
        if (Enum.IsDefined(typeof(LifecycleState), code))
        {
            return (LifecycleState)code;
        }
        throw new ArgumentException($"Invalid state code: {code}", nameof(code));
    }

    /// <summary>
    /// Check if the plugin can handle requests in this state.
    /// </summary>
    /// <param name="state">The lifecycle state.</param>
    /// <returns>True if requests can be handled.</returns>
    public static bool CanHandleRequests(this LifecycleState state) => state == LifecycleState.Active;

    /// <summary>
    /// Check if this is a terminal state.
    /// </summary>
    /// <param name="state">The lifecycle state.</param>
    /// <returns>True if the plugin cannot transition to other states.</returns>
    public static bool IsTerminal(this LifecycleState state) =>
        state == LifecycleState.Stopped || state == LifecycleState.Failed;
}
