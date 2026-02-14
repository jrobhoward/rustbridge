-ifndef(RUSTBRIDGE_HRL).
-define(RUSTBRIDGE_HRL, true).

-record(plugin_config, {
    log_level = info :: rustbridge_log:level(),
    worker_threads :: pos_integer() | undefined,
    max_concurrent_ops = 1000 :: non_neg_integer(),
    shutdown_timeout_ms = 5000 :: non_neg_integer(),
    data = #{} :: map(),
    init_params :: map() | undefined
}).

-record(log_entry, {
    level :: rustbridge_log:level(),
    target :: binary(),
    message :: binary()
}).

-type plugin_ref() :: pid() | atom().
-type type_tag() :: binary() | string().
-type call_result() :: {ok, binary()} | {error, {integer(), binary()}}.
-type raw_result() :: {ok, binary()} | {error, {integer(), binary()}}.
-type lifecycle_state() :: installed | starting | active | stopping | stopped | failed.

-endif.
