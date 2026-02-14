-module(rustbridge_config).

-include("rustbridge.hrl").

-export([defaults/0, to_json_map/1]).

%% @doc Return a default plugin config record.
-spec defaults() -> #plugin_config{}.
defaults() ->
    #plugin_config{}.

%% @doc Convert a plugin_config record to a JSON-serializable map.
-spec to_json_map(#plugin_config{}) -> map().
to_json_map(#plugin_config{
    log_level = LogLevel,
    worker_threads = WorkerThreads,
    max_concurrent_ops = MaxOps,
    shutdown_timeout_ms = ShutdownTimeout,
    data = Data,
    init_params = InitParams
}) ->
    Base = #{
        <<"log_level">> => atom_to_binary(LogLevel, utf8),
        <<"max_concurrent_ops">> => MaxOps,
        <<"shutdown_timeout_ms">> => ShutdownTimeout,
        <<"data">> => Data
    },
    WithThreads =
        case WorkerThreads of
            undefined -> Base;
            N -> Base#{<<"worker_threads">> => N}
        end,
    case InitParams of
        undefined -> WithThreads;
        Params -> WithThreads#{<<"init_params">> => Params}
    end.
