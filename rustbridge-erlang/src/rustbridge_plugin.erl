-module(rustbridge_plugin).
-behaviour(gen_server).

-include("rustbridge.hrl").

%% Public API
-export([
    start_link/2,
    start_link/3,
    start_link/4,
    start_link_bundle/2,
    start_link_bundle/3,
    start_link_bundle/4,
    call/3,
    call/4,
    call_raw/3,
    call_raw/4,
    get_state/1,
    get_rejected_count/1,
    set_log_level/2,
    shutdown/1,
    stop/1
]).

%% gen_server callbacks
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2]).

-record(state, {
    port :: port() | undefined,
    next_id = 1 :: pos_integer(),
    pending = #{} :: #{pos_integer() => {pid(), reference()}},
    log_handler :: fun((#log_entry{}) -> ok) | undefined
}).

%% ---------------------------------------------------------------------------
%% Public API
%% ---------------------------------------------------------------------------

%% @doc Start a plugin from a shared library path.
-spec start_link(string() | binary(), #plugin_config{} | map()) ->
    {ok, pid()} | {error, term()}.
start_link(Path, Config) ->
    gen_server:start_link(?MODULE, {path, Path, Config, #{}}, []).

%% @doc Start a named plugin from a shared library path.
-spec start_link(atom(), string() | binary(), #plugin_config{} | map()) ->
    {ok, pid()} | {error, term()}.
start_link(Name, Path, Config) ->
    gen_server:start_link({local, Name}, ?MODULE, {path, Path, Config, #{}}, []).

%% @doc Start a named plugin with options (e.g., log_handler).
-spec start_link(atom(), string() | binary(), #plugin_config{} | map(), map()) ->
    {ok, pid()} | {error, term()}.
start_link(Name, Path, Config, Opts) ->
    gen_server:start_link({local, Name}, ?MODULE, {path, Path, Config, Opts}, []).

%% @doc Start a plugin from a .rbp bundle.
-spec start_link_bundle(string() | binary(), #plugin_config{} | map()) ->
    {ok, pid()} | {error, term()}.
start_link_bundle(BundlePath, Config) ->
    gen_server:start_link(?MODULE, {bundle, BundlePath, Config, #{}}, []).

%% @doc Start a named plugin from a .rbp bundle.
-spec start_link_bundle(atom(), string() | binary(), #plugin_config{} | map()) ->
    {ok, pid()} | {error, term()}.
start_link_bundle(Name, BundlePath, Config) ->
    gen_server:start_link({local, Name}, ?MODULE, {bundle, BundlePath, Config, #{}}, []).

%% @doc Start a named plugin from a .rbp bundle with options.
-spec start_link_bundle(atom(), string() | binary(), #plugin_config{} | map(), map()) ->
    {ok, pid()} | {error, term()}.
start_link_bundle(Name, BundlePath, Config, Opts) ->
    gen_server:start_link({local, Name}, ?MODULE, {bundle, BundlePath, Config, Opts}, []).

%% @doc Make a JSON transport call to the plugin (5 second timeout).
-spec call(plugin_ref(), type_tag(), binary()) -> call_result().
call(PluginRef, TypeTag, Request) ->
    call(PluginRef, TypeTag, Request, 5000).

%% @doc Make a JSON transport call with explicit timeout.
-spec call(plugin_ref(), type_tag(), binary(), timeout()) -> call_result().
call(PluginRef, TypeTag, Request, Timeout) ->
    gen_server:call(PluginRef, {call, TypeTag, Request}, Timeout).

%% @doc Make a binary transport call (5 second timeout).
-spec call_raw(plugin_ref(), non_neg_integer(), binary()) -> raw_result().
call_raw(PluginRef, MessageId, Data) ->
    call_raw(PluginRef, MessageId, Data, 5000).

%% @doc Make a binary transport call with explicit timeout.
-spec call_raw(plugin_ref(), non_neg_integer(), binary(), timeout()) -> raw_result().
call_raw(PluginRef, MessageId, Data, Timeout) ->
    gen_server:call(PluginRef, {call_raw, MessageId, Data}, Timeout).

%% @doc Get the plugin lifecycle state.
-spec get_state(plugin_ref()) -> {ok, lifecycle_state()} | {error, term()}.
get_state(PluginRef) ->
    case gen_server:call(PluginRef, get_state) of
        {ok, StateBin} when is_binary(StateBin) ->
            {ok, binary_to_existing_atom(StateBin, utf8)};
        {error, _} = Err ->
            Err
    end.

%% @doc Get the number of requests rejected due to concurrency limits.
-spec get_rejected_count(plugin_ref()) -> {ok, non_neg_integer()} | {error, term()}.
get_rejected_count(PluginRef) ->
    gen_server:call(PluginRef, get_rejected_count).

%% @doc Set the plugin log level.
-spec set_log_level(plugin_ref(), rustbridge_log:level()) -> ok | {error, term()}.
set_log_level(PluginRef, Level) ->
    case gen_server:call(PluginRef, {set_log_level, Level}) of
        {ok, _} -> ok;
        {error, _} = Err -> Err
    end.

%% @doc Shutdown the plugin gracefully.
-spec shutdown(plugin_ref()) -> ok | {error, term()}.
shutdown(PluginRef) ->
    case gen_server:call(PluginRef, shutdown) of
        {ok, _} -> ok;
        {error, _} = Err -> Err
    end.

%% @doc Stop the gen_server (sends shutdown first).
-spec stop(plugin_ref()) -> ok.
stop(PluginRef) ->
    gen_server:stop(PluginRef).

%% ---------------------------------------------------------------------------
%% gen_server callbacks
%% ---------------------------------------------------------------------------

init({path, Path, Config, Opts}) ->
    LogHandler = maps:get(log_handler, Opts, undefined),
    ConfigMap = to_config_map(Config),
    Port = open_driver_port(),
    State = #state{port = Port, log_handler = LogHandler},
    case
        send_and_wait(State, fun(Id) -> rustbridge_protocol:encode_load(Id, Path, ConfigMap) end)
    of
        {ok, _Data, State1} ->
            {ok, State1};
        {error, Reason, _State1} ->
            port_close(Port),
            {stop, Reason}
    end;
init({bundle, BundlePath, Config, Opts}) ->
    LogHandler = maps:get(log_handler, Opts, undefined),
    ConfigMap = to_config_map(Config),
    BundleOpts = maps:with([verify_signatures, public_key], Opts),
    Port = open_driver_port(),
    State = #state{port = Port, log_handler = LogHandler},
    case
        send_and_wait(State, fun(Id) ->
            rustbridge_protocol:encode_load_bundle(Id, BundlePath, ConfigMap, BundleOpts)
        end)
    of
        {ok, _Data, State1} ->
            {ok, State1};
        {error, Reason, _State1} ->
            port_close(Port),
            {stop, Reason}
    end.

handle_call({call, TypeTag, Request}, From, State) ->
    {Id, State1} = next_id(State),
    Frame = rustbridge_protocol:encode_call(Id, TypeTag, Request),
    send_to_port(State1#state.port, Frame),
    State2 = add_pending(State1, Id, From),
    {noreply, State2};
handle_call({call_raw, MessageId, Data}, From, State) ->
    {Id, State1} = next_id(State),
    Frame = rustbridge_protocol:encode_call_raw(Id, MessageId, Data),
    send_to_port(State1#state.port, Frame),
    State2 = add_pending(State1, Id, From),
    {noreply, State2};
handle_call(get_state, From, State) ->
    {Id, State1} = next_id(State),
    Frame = rustbridge_protocol:encode_get_state(Id),
    send_to_port(State1#state.port, Frame),
    State2 = add_pending(State1, Id, From),
    {noreply, State2};
handle_call(get_rejected_count, From, State) ->
    {Id, State1} = next_id(State),
    Frame = rustbridge_protocol:encode_get_rejected_count(Id),
    send_to_port(State1#state.port, Frame),
    State2 = add_pending(State1, Id, From),
    {noreply, State2};
handle_call({set_log_level, Level}, From, State) ->
    {Id, State1} = next_id(State),
    LevelCode = rustbridge_log:to_code(Level),
    Frame = rustbridge_protocol:encode_set_log_level(Id, LevelCode),
    send_to_port(State1#state.port, Frame),
    State2 = add_pending(State1, Id, From),
    {noreply, State2};
handle_call(shutdown, From, State) ->
    {Id, State1} = next_id(State),
    Frame = rustbridge_protocol:encode_shutdown(Id),
    send_to_port(State1#state.port, Frame),
    State2 = add_pending(State1, Id, From),
    {noreply, State2}.

handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info({Port, {data, Data}}, #state{port = Port} = State) ->
    case rustbridge_protocol:decode_message(Data) of
        {response, Id, Result} ->
            State1 = reply_pending(State, Id, Result),
            {noreply, State1};
        {log, LogEntry} ->
            handle_log(State, LogEntry),
            {noreply, State}
    end;
handle_info({Port, {exit_status, _Code}}, #state{port = Port} = State) ->
    %% Port driver crashed - fail all pending callers
    State1 = fail_all_pending(State, {error, port_crashed}),
    {stop, port_crashed, State1#state{port = undefined}};
handle_info(_Info, State) ->
    {noreply, State}.

terminate(_Reason, #state{port = undefined}) ->
    ok;
terminate(_Reason, #state{port = Port}) ->
    %% Try to send shutdown, but don't wait for response
    catch port_close(Port),
    ok.

%% ---------------------------------------------------------------------------
%% Internal helpers
%% ---------------------------------------------------------------------------

open_driver_port() ->
    DriverPath = driver_path(),
    open_port(
        {spawn_executable, DriverPath},
        [{packet, 4}, binary, exit_status, use_stdio]
    ).

driver_path() ->
    PrivDir = code:priv_dir(rustbridge),
    filename:join(PrivDir, "rustbridge-port-driver").

next_id(#state{next_id = Id} = State) ->
    {Id, State#state{next_id = Id + 1}}.

add_pending(#state{pending = Pending} = State, Id, From) ->
    State#state{pending = Pending#{Id => From}}.

reply_pending(#state{pending = Pending} = State, Id, Result) ->
    case maps:take(Id, Pending) of
        {From, Pending1} ->
            gen_server:reply(From, Result),
            State#state{pending = Pending1};
        error ->
            %% Unexpected response id, ignore
            State
    end.

fail_all_pending(#state{pending = Pending} = State, Error) ->
    maps:foreach(fun(_Id, From) -> gen_server:reply(From, Error) end, Pending),
    State#state{pending = #{}}.

send_to_port(Port, Data) ->
    port_command(Port, Data).

%% Synchronous send-and-wait used only during init.
send_and_wait(State, EncodeFun) ->
    {Id, State1} = next_id(State),
    Frame = EncodeFun(Id),
    send_to_port(State1#state.port, Frame),
    receive
        {Port, {data, RespData}} when Port =:= State1#state.port ->
            case rustbridge_protocol:decode_message(RespData) of
                {response, Id, {ok, Data}} ->
                    {ok, Data, State1};
                {response, Id, {error, Reason}} ->
                    {error, Reason, State1};
                {log, LogEntry} ->
                    handle_log(State1, LogEntry),
                    %% Keep waiting for the actual response
                    wait_for_response(State1, Id)
            end;
        {Port, {exit_status, Code}} when Port =:= State1#state.port ->
            {error, {port_exit, Code}, State1}
    after 30000 ->
        {error, init_timeout, State1}
    end.

wait_for_response(State, Id) ->
    receive
        {Port, {data, RespData}} when Port =:= State#state.port ->
            case rustbridge_protocol:decode_message(RespData) of
                {response, Id, {ok, Data}} ->
                    {ok, Data, State};
                {response, Id, {error, Reason}} ->
                    {error, Reason, State};
                {log, LogEntry} ->
                    handle_log(State, LogEntry),
                    wait_for_response(State, Id)
            end;
        {Port, {exit_status, Code}} when Port =:= State#state.port ->
            {error, {port_exit, Code}, State}
    after 30000 ->
        {error, init_timeout, State}
    end.

handle_log(#state{log_handler = undefined}, #log_entry{
    level = Level, target = Target, message = Message
}) ->
    %% Default: route to OTP logger
    LoggerLevel = rustbridge_log:to_logger_level(Level),
    logger:log(LoggerLevel, Message, #{domain => [rustbridge], target => Target});
handle_log(#state{log_handler = Handler}, LogEntry) ->
    %% Custom handler
    try
        Handler(LogEntry)
    catch
        _:_ -> ok
    end.

to_config_map(Config) when is_map(Config) ->
    Config;
to_config_map(Config) when is_tuple(Config), element(1, Config) =:= plugin_config ->
    rustbridge_config:to_json_map(Config).
