# Getting Started: Erlang

This guide walks you through using rustbridge plugins from Erlang/OTP using the port-based driver.

## Prerequisites

- **Erlang/OTP 27 or later** - For modern maps, json module, and logger support
  ```bash
  erl -eval 'io:format("~s~n", [erlang:system_info(otp_release)]), halt().' -noshell
  ```
- **rebar3** - The standard Erlang build tool
- **Rust toolchain** - Required to compile the port driver binary (>= 1.90.0)
- **A rustbridge plugin** - Either a `.rbp` bundle or native library

## Installation

### Using rebar3

Add `rustbridge` as a dependency in your `rebar.config`:

```erlang
{deps, [
    {rustbridge, {git, "https://github.com/example/rustbridge.git", {branch, "main"}}}
]}.
```

Then fetch and compile. The build hooks automatically compile the Rust port driver and place it in `priv/`:

```bash
rebar3 compile
```

### Local Development

When working with rustbridge source code directly:

```bash
cd rustbridge-erlang
rebar3 compile
```

The pre-hooks in `rebar.config` handle building the Rust port driver automatically. The test profile also builds the example `hello-plugin`:

```bash
rebar3 ct
```

## Architecture Note

Unlike the Python, Java, and C# integrations that load the Rust shared library directly via FFI, the Erlang integration uses an **Erlang port** (external process) architecture.

The BEAM virtual machine has strict scheduling requirements. A NIF that blocks too long starves the BEAM scheduler, degrading the entire node. Rust plugins run an async Tokio runtime with potentially long-running operations, making NIF integration risky. The port-based approach provides:

- **Fault isolation** - If the plugin crashes, only the port process dies. The supervisor can restart it automatically.
- **Scheduler safety** - The BEAM scheduler is never blocked by plugin work.
- **OTP compatibility** - `rustbridge_plugin` is a standard `gen_server`, fitting naturally into supervision trees.

The trade-off is IPC overhead (roughly 20-30 microseconds per call) and data copying between the BEAM and port process. For most applications, the fault isolation benefits far outweigh this cost.

## Loading a Plugin

### From a Library Path

```erlang
-include("rustbridge.hrl").

%% Start an unnamed plugin
{ok, Plugin} = rustbridge_plugin:start_link(
    "target/release/libmyplugin.so", #plugin_config{}).

%% Start a named plugin (registered process name)
{ok, Plugin} = rustbridge_plugin:start_link(
    my_plugin, "target/release/libmyplugin.so", #plugin_config{}).
```

Platform-specific library paths:

```erlang
LibPath = case os:type() of
    {unix, darwin} -> "target/release/libmyplugin.dylib";
    {unix, _}      -> "target/release/libmyplugin.so";
    {win32, _}     -> "target/release/myplugin.dll"
end,
{ok, Plugin} = rustbridge_plugin:start_link(LibPath, #plugin_config{}).
```

### From a Bundle

```erlang
%% Load from a .rbp bundle
{ok, Plugin} = rustbridge_plugin:start_link_bundle(
    "my-plugin-1.0.0.rbp", #plugin_config{}).

%% Named bundle plugin with signature verification
{ok, Plugin} = rustbridge_plugin:start_link_bundle(
    my_plugin, "my-plugin-1.0.0.rbp", #plugin_config{},
    #{verify_signatures => true, public_key => <<"RWQ...">>}).
```

### Under a Supervisor

Use `rustbridge_sup` to manage plugin lifecycles within an OTP supervision tree:

```erlang
application:ensure_all_started(rustbridge),
{ok, Pid} = rustbridge_sup:start_plugin(my_plugin, LibPath, #plugin_config{}),

%% Later, stop it
ok = rustbridge_sup:stop_plugin(my_plugin).
```

The supervisor uses a `one_for_one` strategy with `transient` restart, so a plugin that exits abnormally will be restarted automatically.

## Making JSON Calls

The `call/3` and `call/4` functions send a JSON request and return the response as a binary:

```erlang
%% Simple call (default 5 second timeout)
{ok, Response} = rustbridge_plugin:call(
    Plugin, <<"echo">>, <<"{\"message\": \"Hello from Erlang!\"}">>),

Map = json:decode(Response),
Message = maps:get(<<"message">>, Map),
Length = maps:get(<<"length">>, Map).
```

```erlang
%% With explicit timeout and maps-based request building
Request = json:encode(#{<<"a">> => 42, <<"b">> => 58}),
{ok, Response} = rustbridge_plugin:call(Plugin, <<"math.add">>, Request, 10000),

Map = json:decode(Response),
100 = maps:get(<<"result">>, Map).
```

For performance-critical paths, use binary transport via `call_raw/3`:

```erlang
MessageId = 1,
RequestBin = <<1:8, 0:24,                       %% version + reserved
               "bench_key", 0:(55*8),            %% 64-byte key field
               9:32/native, 16#01:32/native>>,   %% key_len + flags

{ok, ResponseBin} = rustbridge_plugin:call_raw(Plugin, MessageId, RequestBin).
```

## Configuration

Include `rustbridge.hrl` to use the `#plugin_config{}` record:

```erlang
-include("rustbridge.hrl").

Config = #plugin_config{
    log_level          = debug,           %% trace | debug | info | warn | error | off
    worker_threads     = 4,               %% Tokio worker threads (undefined = Rust default)
    max_concurrent_ops = 500,             %% Max in-flight operations (default: 1000)
    shutdown_timeout_ms = 10000,          %% Graceful shutdown timeout (default: 5000)
    data               = #{},             %% Arbitrary data passed to the plugin
    init_params        = #{               %% Plugin-specific init parameters
        <<"db_url">> => <<"postgres://localhost/mydb">>
    }
},

{ok, Plugin} = rustbridge_plugin:start_link(LibPath, Config).
```

You can also pass configuration as a plain map:

```erlang
Config = #{<<"log_level">> => <<"info">>, <<"worker_threads">> => 4}.
```

## Logging

By default, log messages from the Rust plugin are routed to the OTP `logger` with the `[rustbridge]` domain. Rust log levels map to OTP levels: `trace`/`debug` to `debug`, `info` to `info`, `warn` to `warning`, `error` to `error`.

### Custom Log Handler

Pass a log handler function in the options map:

```erlang
-include("rustbridge.hrl").

LogHandler = fun(#log_entry{level = Level, target = Target, message = Msg}) ->
    io:format("[~s] ~s: ~s~n", [Level, Target, Msg]),
    ok
end,

{ok, Plugin} = rustbridge_plugin:start_link(
    my_plugin, LibPath, #plugin_config{log_level = trace},
    #{log_handler => LogHandler}).
```

Change the log level at runtime:

```erlang
ok = rustbridge_plugin:set_log_level(Plugin, debug).
```

## Error Handling

Plugin calls return tagged tuples for pattern matching:

```erlang
case rustbridge_plugin:call(Plugin, TypeTag, Request) of
    {ok, Response} ->
        {ok, json:decode(Response)};
    {error, {6, _Msg}} ->
        {error, unknown_type};        %% Unknown message type
    {error, {7, Msg}} ->
        {error, {handler_error, Msg}};  %% Plugin-defined error
    {error, {13, _Msg}} ->
        {error, overloaded};           %% Too many concurrent requests
    {error, {Code, Msg}} ->
        {error, {unexpected, Code, Msg}}
end.
```

Handle timeouts and port failures with try/catch:

```erlang
try rustbridge_plugin:call(Plugin, <<"slow.op">>, <<"{}">>, 30000)
catch
    exit:{timeout, _} -> {error, timeout};
    exit:{noproc, _}  -> {error, not_running}
end.
```

## Concurrent Usage

Since `rustbridge_plugin` is a `gen_server`, it handles concurrent calls from multiple OTP processes naturally:

```erlang
{ok, _} = rustbridge_plugin:start_link(
    my_plugin, LibPath, #plugin_config{max_concurrent_ops = 500}),

Self = self(),
Pids = [spawn_link(fun() ->
    Request = json:encode(#{<<"message">> => <<"concurrent">>}),
    Result = rustbridge_plugin:call(my_plugin, <<"echo">>, Request, 10000),
    Self ! {done, self(), Result}
end) || _ <- lists:seq(1, 10)],

Results = [receive {done, Pid, R} -> R after 15000 -> timeout end || Pid <- Pids],
OkCount = length([ok || {ok, _} <- Results]),
io:format("Completed ~B/~B calls~n", [OkCount, length(Results)]).
```

## Monitoring

```erlang
%% Check plugin lifecycle state
{ok, State} = rustbridge_plugin:get_state(Plugin),
%% Possible states: installed | starting | active | stopping | stopped | failed

%% Monitor the plugin process (standard OTP)
Ref = monitor(process, Plugin),
receive {'DOWN', Ref, process, Plugin, Reason} ->
    io:format("Plugin exited: ~p~n", [Reason])
after 0 -> ok end.

%% Graceful shutdown: shutdown first, then stop
ok = rustbridge_plugin:shutdown(Plugin),
ok = rustbridge_plugin:stop(Plugin).
```

## Complete Example

```erlang
-module(my_app_worker).
-behaviour(gen_server).
-include("rustbridge.hrl").

-export([start_link/1, add/3, echo/2]).
-export([init/1, handle_call/3, handle_cast/2, terminate/2]).

-record(state, {plugin :: pid()}).

start_link(LibPath) ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, LibPath, []).

add(A, B, Timeout) ->
    gen_server:call(?MODULE, {add, A, B}, Timeout).

echo(Message, Timeout) ->
    gen_server:call(?MODULE, {echo, Message}, Timeout).

init(LibPath) ->
    LogHandler = fun(#log_entry{level = Level, message = Msg}) ->
        logger:log(rustbridge_log:to_logger_level(Level),
                   "~s", [Msg], #{domain => [my_app, plugin]})
    end,
    Config = #plugin_config{
        log_level = info, worker_threads = 4, shutdown_timeout_ms = 10000
    },
    case rustbridge_plugin:start_link(
            calc_plugin, LibPath, Config, #{log_handler => LogHandler}) of
        {ok, Plugin} ->
            {ok, active} = rustbridge_plugin:get_state(Plugin),
            {ok, #state{plugin = Plugin}};
        {error, Reason} ->
            {stop, Reason}
    end.

handle_call({add, A, B}, _From, #state{plugin = Plugin} = State) ->
    Request = json:encode(#{<<"a">> => A, <<"b">> => B}),
    case rustbridge_plugin:call(Plugin, <<"math.add">>, Request) of
        {ok, Resp} -> {reply, {ok, maps:get(<<"result">>, json:decode(Resp))}, State};
        {error, _} = Err -> {reply, Err, State}
    end;
handle_call({echo, Message}, _From, #state{plugin = Plugin} = State) ->
    Request = json:encode(#{<<"message">> => Message}),
    case rustbridge_plugin:call(Plugin, <<"echo">>, Request) of
        {ok, Resp} -> {reply, {ok, json:decode(Resp)}, State};
        {error, _} = Err -> {reply, Err, State}
    end.

handle_cast(_Msg, State) -> {noreply, State}.

terminate(_Reason, #state{plugin = Plugin}) ->
    rustbridge_plugin:shutdown(Plugin),
    rustbridge_plugin:stop(Plugin),
    ok.
```

Usage from the shell:

```erlang
1> application:ensure_all_started(rustbridge).
{ok, [rustbridge]}
2> my_app_worker:start_link("target/release/libcalculator_plugin.so").
{ok, <0.150.0>}
3> my_app_worker:add(42, 58, 5000).
{ok, 100}
4> my_app_worker:echo(<<"Hello!">>, 5000).
{ok, #{<<"message">> => <<"Hello!">>, <<"length">> => 6}}
```

## Performance Notes

The Erlang integration uses port-based IPC, so latencies are higher than in-process FFI integrations. The trade-off is fault isolation and BEAM scheduler safety.

| Transport   | Latency (Linux) | Notes                             |
|-------------|-----------------|-----------------------------------|
| Port JSON   | 36.7 us         | JSON encode/decode + IPC overhead |
| Port Binary | 25.6 us         | Binary protocol + IPC overhead    |

Binary transport is approximately **1.4x faster** than JSON over the port. The gap is smaller than in FFI-based integrations because IPC overhead dominates.

For performance-critical applications, consider:
- Using binary transport to reduce serialization cost
- Batching multiple operations into a single plugin call
- Adjusting `max_concurrent_ops` to match your workload

## Testing

```bash
# Run all Common Test suites (builds port driver and hello-plugin automatically)
cd rustbridge-erlang
rebar3 ct

# Run specific suite or test case
rebar3 ct --suite=rustbridge_plugin_SUITE
rebar3 ct --suite=rustbridge_plugin_SUITE --case=call___echo_message___returns_response

# Run EUnit tests
rebar3 eunit

# Run benchmarks
rebar3 ct --suite=rustbridge_bench_SUITE
```

Tests follow the project-wide triple-underscore naming convention: `subjectUnderTest___condition___expectedResult`. For example:

```erlang
call___echo_message___returns_response(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, Response} = rustbridge_plugin:call(
        Plugin, <<"echo">>, <<"{\"message\": \"Hello from Erlang!\"}">>),

    Map = json:decode(Response),
    <<"Hello from Erlang!">> = maps:get(<<"message">>, Map).
```

Before running integration tests, the hello-plugin native library must be built. The rebar3 test profile pre-hooks handle this automatically, but you can also build it manually:

```bash
cargo build --release -p hello-plugin
```

## Related Documentation

- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details (JSON and binary)
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership patterns
- [../ERROR_HANDLING.md](../ERROR_HANDLING.md) - Error codes and handling patterns
- [../ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture overview
- [../PLUGIN_LIFECYCLE.md](../PLUGIN_LIFECYCLE.md) - Plugin lifecycle and resource cleanup
