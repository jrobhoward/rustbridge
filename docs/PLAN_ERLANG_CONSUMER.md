# Plan: Erlang Consumer for rustbridge

## Context

rustbridge supports Java, C#, Python, and Rust consumers. This plan adds Erlang as a consumer language using the **Port** approach: a small Rust binary (the "port driver") loads the `.rbp` plugin and communicates with the BEAM VM over stdin/stdout. This gives crash isolation (plugin crash doesn't take down the VM) and clean OTP integration at the cost of some IPC overhead vs a NIF approach.

The implementation has two halves:
1. **Rust side** (`crates/rustbridge-port-driver/`) - a binary that wraps `rustbridge-consumer`, reads commands from stdin, and writes responses to stdout
2. **Erlang side** (`rustbridge-erlang/`) - a rebar3 OTP application with a gen_server wrapping the port

## Architecture

```
Erlang process                                          Plugin (.so)
     |                                                       |
     |-- gen_server:call(Plugin, {call, "echo", Json}) -->   |
     |                                                       |
     |   rustbridge_plugin (gen_server)                      |
     |     |                                                 |
     |     |-- encode JSON command -->                       |
     |     |-- port_command(Port, Frame) -->                 |
     |     |                                                 |
     |     |   rustbridge-port-driver (Rust binary, stdio)   |
     |     |     |                                           |
     |     |     |-- NativePlugin::call("echo", json) ------>|
     |     |     |<--- FfiBuffer (JSON response) ------------|
     |     |     |                                           |
     |     |     |-- encode JSON response -->                |
     |     |<--- {packet,4} frame ---------------------------|
     |     |                                                 |
     |<--- {ok, ResponseJson} -------------------------------|
```

## Wire Protocol

**Framing**: Erlang `{packet, 4}` (4-byte big-endian length prefix). All payloads are JSON.

**Commands** (Erlang → Rust), keyed by `"type"`:

| type | fields | description |
|------|--------|-------------|
| `load` | id, path, config? | Load plugin from .so/.dylib path |
| `load_bundle` | id, path, config?, verify_signatures?, public_key? | Load from .rbp bundle |
| `call` | id, type_tag, payload | JSON transport call |
| `call_raw` | id, message_id, data (base64) | Binary transport call |
| `get_state` | id | Query lifecycle state |
| `set_log_level` | id, level (u8) | Change log level |
| `shutdown` | id | Graceful shutdown |

**Responses** (Rust → Erlang):

```json
{"type": "response", "id": 3, "status": "ok", "data": "..."}
{"type": "response", "id": 3, "status": "error", "error_code": 6, "error_message": "..."}
```

**Log messages** (Rust → Erlang, unsolicited, no `id`):

```json
{"type": "log", "level": 2, "target": "hello_plugin", "message": "Handling echo"}
```

The gen_server dispatches by `"type"`: `"response"` → match pending caller by `"id"`, `"log"` → route to log handler.

**Binary transport**: `call_raw` data is base64-encoded in the JSON envelope. This avoids a mixed binary/JSON protocol while preserving the benefit of skipping JSON serialization inside the plugin.

## Log Routing

Plugin log messages arrive as unsolicited `"log"` messages from the port driver. The gen_server routes them to OTP `logger` by default, with an optional user-provided callback override.

**Default behavior** (OTP `logger`): The gen_server calls `logger:log/3` with metadata including the plugin target module:

```erlang
logger:log(Level, Message, #{domain => [rustbridge], target => Target})
```

This integrates with OTP's standard log handlers, filters, and formatters. Users can filter rustbridge logs via `logger:set_module_level/2` or domain-based filters.

**Custom callback override**: Pass a fun when starting the plugin:

```erlang
LogHandler = fun(Level, Target, Message) -> io:format("[~p] ~s: ~s~n", [Level, Target, Message]) end,
rustbridge_plugin:start_link(Path, Config, #{log_handler => LogHandler}).
```

**Level mapping** (Rust FFI codes → OTP logger levels):

| Rust code | Rust level | OTP logger level |
|-----------|-----------|-----------------|
| 0 | Trace | `debug` (OTP has no trace) |
| 1 | Debug | `debug` |
| 2 | Info | `info` |
| 3 | Warn | `warning` |
| 4 | Error | `error` |

The `rustbridge_log` module handles this mapping via `to_logger_level/1`.

## Implementation

### Phase 1: Rust Port Driver (`crates/rustbridge-port-driver/`)

New workspace crate producing a binary.

**Files to create:**

- `crates/rustbridge-port-driver/Cargo.toml` - depends on `rustbridge-consumer`, `serde`, `serde_json`, `base64`
- `crates/rustbridge-port-driver/src/main.rs` - stdio read/write loop with `{packet, 4}` framing; spawns a log-writer thread for async log forwarding; all stdout writes protected by a mutex
- `crates/rustbridge-port-driver/src/protocol.rs` - serde types for Command (tagged enum), Response, LogMessage
- `crates/rustbridge-port-driver/src/handler.rs` - holds `Option<NativePlugin>`, dispatches commands to `rustbridge-consumer` API
- `crates/rustbridge-port-driver/src/error.rs` - port-driver-specific error types (codes 200+)

**Key file to reference:**
- `crates/rustbridge-consumer/src/plugin.rs` - NativePlugin API (call, call_raw, state, shutdown, set_log_level)
- `crates/rustbridge-consumer/src/loader.rs` - NativePluginLoader (load, load_bundle, log callback setup)

**Error codes**: Plugin errors pass through (1-13). Port-driver errors use 200+ range: 200=protocol parse error, 201=plugin not loaded, 202=plugin already loaded, 203=base64 decode error.

**Add to root `Cargo.toml`** workspace members list.

### Phase 2: Erlang OTP Application (`rustbridge-erlang/`)

Bootstrap with `rebar3 new app rustbridge`. Uses OTP 27+ built-in `json` module (no deps).

**Files to create:**

- `rustbridge-erlang/rebar.config` - pre-hooks to `cargo build --release -p rustbridge-port-driver` and copy to `priv/`
- `rustbridge-erlang/include/rustbridge.hrl` - records (`#plugin_config{}`, `#log_entry{}`) and type specs
- `rustbridge-erlang/src/rustbridge.app.src` - OTP application descriptor
- `rustbridge-erlang/src/rustbridge_app.erl` - application behaviour (starts supervisor)
- `rustbridge-erlang/src/rustbridge_sup.erl` - dynamic supervisor (`one_for_one`), with `start_plugin/3`
- `rustbridge-erlang/src/rustbridge_plugin.erl` - **main API**, gen_server wrapping the port:
  - `start_link/2`, `start_link/3` (with name), `start_link_bundle/2`
  - `call/3`, `call/4` (with timeout) → `{ok, binary()} | {error, {integer(), binary()}}`
  - `call_raw/3`, `call_raw/4` → same
  - `get_state/1` → `{ok, lifecycle_state()}`
  - `set_log_level/2` → `ok`
  - `shutdown/1`, `stop/1`
  - State: `port, next_id, pending :: #{id => from}, log_handler`
- `rustbridge-erlang/src/rustbridge_protocol.erl` - encode commands / decode responses / decode log messages
- `rustbridge-erlang/src/rustbridge_config.erl` - config builder, `to_json_map/1`
- `rustbridge-erlang/src/rustbridge_log.erl` - log level type, `to_code/1`, `from_code/1`

### Phase 3: Tests

**EUnit** (protocol, config, log level - no port needed):

- `rustbridge-erlang/test/rustbridge_protocol_tests.erl`
- `rustbridge-erlang/test/rustbridge_config_tests.erl`
- `rustbridge-erlang/test/rustbridge_log_tests.erl`

**Common Test** (integration against hello-plugin):

- `rustbridge-erlang/test/rustbridge_plugin_SUITE.erl`

Test scenarios (matching other consumers, triple-underscore naming):
- `plugin___default_config___is_active`
- `call___echo_message___returns_response`
- `call___greet___returns_greeting`
- `call___user_create___returns_user_id`
- `call___math_add___returns_sum`
- `call___unknown_type___returns_error_code_6`
- `get_state___after_load___returns_active`
- `shutdown___explicit___state_becomes_stopped`
- `call_raw___small_request___returns_response`
- `log_callback___receives_log_messages`
- `concurrent_calls___multiple_processes___all_succeed`

CT `init_per_suite` builds hello-plugin and port driver. `init_per_testcase` starts a fresh plugin. `end_per_testcase` stops it.

### Phase 4: Build Integration & Docs

- Update `scripts/pre-commit.sh` - add Erlang change detection and `rebar3 eunit && rebar3 ct` step
- Update `.github/workflows/ci.yml` - add `erlef/setup-beam` action, run Erlang tests
- Create `docs/TESTING_ERLANG.md` following existing pattern
- Update `CLAUDE.md` with Erlang commands section

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Port vs NIF | Port | Crash isolation, simpler FFI, OTP-idiomatic; user preference |
| Wire format | JSON over {packet,4} | Debuggable, consistent with existing transport, avoids ETF parser in Rust |
| Binary transport encoding | Base64 in JSON | Simple; the perf benefit is in the plugin (no JSON serde), not the wire |
| One port per plugin | Yes | Maps naturally to gen_server/supervisor; pool can be added later |
| JSON library | OTP 27 built-in `json` | No external deps needed; user confirmed OTP 27+ |
| Command processing | Single-threaded | gen_server already serializes; port IPC is the bottleneck, not CPU |
| Log forwarding | Background thread + mutex | Logs are async/unsolicited; mutex prevents stdout interleaving |
| Log routing | OTP `logger` by default | Idiomatic OTP integration; optional custom callback for flexibility |

## Verification

1. `cargo build --release -p rustbridge-port-driver` - port driver compiles
2. `cargo test -p rustbridge-port-driver` - Rust protocol/handler unit tests pass
3. `cd rustbridge-erlang && rebar3 eunit` - protocol/config/log unit tests pass
4. `cd rustbridge-erlang && rebar3 ct` - full integration suite against hello-plugin passes
5. `./scripts/pre-commit.sh --smart` - detects Erlang changes and runs tests
