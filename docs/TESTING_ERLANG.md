# Testing Conventions: Erlang

## Overview

The Erlang consumer uses two test frameworks:
- **EUnit** for fast unit tests (protocol encoding, config, log level mapping)
- **Common Test** for integration tests against the hello-plugin

## Running Tests

```bash
# From rustbridge-erlang/
rebar3 eunit                  # Unit tests only
rebar3 ct                     # Integration tests only
rebar3 ct --verbose           # Integration tests with output
rebar3 eunit && rebar3 ct     # Both
```

Integration tests require the hello-plugin to be built. The rebar3 test profile pre-hook handles this automatically.

## Test Naming Convention

Follow the project-wide triple-underscore convention:

```
subject_under_test___condition___expected_result
```

EUnit test function names end with `_test`:

```erlang
to_code___trace___returns_0_test() ->
    ?assertEqual(0, rustbridge_log:to_code(trace)).
```

Common Test function names match the convention without the `_test` suffix:

```erlang
call___echo_message___returns_response(Config) ->
    Plugin = ?config(plugin, Config),
    {ok, Response} = rustbridge_plugin:call(Plugin, <<"echo">>, <<"{\"message\": \"hello\"}">>),
    Map = json:decode(Response),
    <<"hello">> = maps:get(<<"message">>, Map).
```

## Test Structure

### EUnit Tests (`test/*_tests.erl`)

- `rustbridge_protocol_tests.erl` - Wire protocol encode/decode
- `rustbridge_config_tests.erl` - Plugin config builder
- `rustbridge_log_tests.erl` - Log level conversions

### Common Test Suites (`test/*_SUITE.erl`)

- `rustbridge_plugin_SUITE.erl` - Full integration against hello-plugin

## Common Test Setup

```erlang
init_per_suite(Config) ->
    %% Locate hello-plugin .so/.dylib
    %% Returns {skip, Reason} if not found

init_per_testcase(_TestCase, Config) ->
    %% Start a fresh plugin instance per test
    {ok, Pid} = rustbridge_plugin:start_link(LibPath, #{}),
    [{plugin, Pid} | Config].

end_per_testcase(_TestCase, Config) ->
    %% Stop the plugin
    rustbridge_plugin:stop(?config(plugin, Config)).
```

## Arrange-Act-Assert

Use blank lines to separate sections. No inline comments for sections:

```erlang
call___math_add___returns_sum(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, Response} = rustbridge_plugin:call(Plugin, <<"math.add">>, <<"{\"a\": 10, \"b\": 32}">>),

    Map = json:decode(Response),
    42 = maps:get(<<"result">>, Map).
```

## Error Handling Tests

Errors return `{error, {ErrorCode, ErrorMessage}}`:

```erlang
call___unknown_type___returns_error_code_6(Config) ->
    Plugin = ?config(plugin, Config),

    {error, {6, _Message}} = rustbridge_plugin:call(Plugin, <<"nonexistent">>, <<"{}">>, 10000).
```

## Dependencies

- OTP 27+ (for built-in `json` module)
- rebar3
- The Rust port driver binary (built automatically by rebar3 pre-hook)
- hello-plugin (built automatically by test profile pre-hook)
