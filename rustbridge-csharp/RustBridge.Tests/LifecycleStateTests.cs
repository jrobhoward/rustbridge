namespace RustBridge.Tests;

/// <summary>
/// Tests for <see cref="LifecycleState"/> enum and extensions.
/// </summary>
public class LifecycleStateTests
{
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
        Assert.Throws<ArgumentException>(() => LifecycleStateExtensions.FromCode(code));
    }

    [Fact]
    public void CanHandleRequests___ActiveState___ReturnsTrue()
    {
        Assert.True(LifecycleState.Active.CanHandleRequests());
    }

    [Theory]
    [InlineData(LifecycleState.Installed)]
    [InlineData(LifecycleState.Starting)]
    [InlineData(LifecycleState.Stopping)]
    [InlineData(LifecycleState.Stopped)]
    [InlineData(LifecycleState.Failed)]
    public void CanHandleRequests___NonActiveState___ReturnsFalse(LifecycleState state)
    {
        Assert.False(state.CanHandleRequests());
    }

    [Theory]
    [InlineData(LifecycleState.Stopped)]
    [InlineData(LifecycleState.Failed)]
    public void IsTerminal___TerminalState___ReturnsTrue(LifecycleState state)
    {
        Assert.True(state.IsTerminal());
    }

    [Theory]
    [InlineData(LifecycleState.Installed)]
    [InlineData(LifecycleState.Starting)]
    [InlineData(LifecycleState.Active)]
    [InlineData(LifecycleState.Stopping)]
    public void IsTerminal___NonTerminalState___ReturnsFalse(LifecycleState state)
    {
        Assert.False(state.IsTerminal());
    }
}
