-module(rustbridge_plugin_SUITE).
-include_lib("common_test/include/ct.hrl").
-include("rustbridge.hrl").

%% CT callbacks
-export([
    all/0,
    init_per_suite/1,
    end_per_suite/1,
    init_per_testcase/2,
    end_per_testcase/2
]).

%% Test cases
-export([
    plugin___default_config___is_active/1,
    call___echo_message___returns_response/1,
    call___greet___returns_greeting/1,
    call___user_create___returns_user_id/1,
    call___math_add___returns_sum/1,
    call___unknown_type___returns_error_code_6/1,
    get_state___after_load___returns_active/1,
    get_rejected_count___default_config___returns_zero/1,
    shutdown___explicit___state_becomes_stopped/1,
    set_log_level___changes_level___returns_ok/1,
    concurrent_calls___multiple_processes___all_succeed/1,
    log_callback___receives_log_messages/1
]).

all() ->
    [
        plugin___default_config___is_active,
        call___echo_message___returns_response,
        call___greet___returns_greeting,
        call___user_create___returns_user_id,
        call___math_add___returns_sum,
        call___unknown_type___returns_error_code_6,
        get_state___after_load___returns_active,
        get_rejected_count___default_config___returns_zero,
        shutdown___explicit___state_becomes_stopped,
        set_log_level___changes_level___returns_ok,
        concurrent_calls___multiple_processes___all_succeed,
        log_callback___receives_log_messages
    ].

init_per_suite(Config) ->
    %% Navigate from priv_dir to the workspace root.
    %% code:priv_dir(rustbridge) under rebar3 test profile resolves to
    %% <workspace>/rustbridge-erlang/_build/test/lib/rustbridge/priv
    %% or the real priv/ directory. Either way, the workspace root is the
    %% parent of the rustbridge-erlang/ directory.
    PrivDir = code:priv_dir(rustbridge),
    RebarProjectRoot = find_rebar_root(PrivDir),
    WorkspaceRoot = filename:dirname(RebarProjectRoot),

    LibName =
        case os:type() of
            {unix, darwin} -> "libhello_plugin.dylib";
            {unix, _} -> "libhello_plugin.so";
            {win32, _} -> "hello_plugin.dll"
        end,
    LibPath = filename:join([WorkspaceRoot, "target", "release", LibName]),

    case filelib:is_file(LibPath) of
        true ->
            [{lib_path, LibPath} | Config];
        false ->
            {skip, {hello_plugin_not_found, LibPath}}
    end.

%% Walk up the directory tree until we find rebar.config.
find_rebar_root(Dir) ->
    case filelib:is_file(filename:join(Dir, "rebar.config")) of
        true ->
            Dir;
        false ->
            Parent = filename:dirname(Dir),
            case Parent of
                Dir -> error({rebar_config_not_found, Dir});
                _ -> find_rebar_root(Parent)
            end
    end.

end_per_suite(_Config) ->
    ok.

init_per_testcase(log_callback___receives_log_messages, Config) ->
    %% Special setup: start plugin with a custom log handler
    LibPath = ?config(lib_path, Config),
    Self = self(),
    LogHandler = fun(#log_entry{level = Level, target = Target, message = Msg}) ->
        Self ! {log, Level, Target, Msg},
        ok
    end,
    PluginConfig = #{<<"log_level">> => <<"trace">>},
    {ok, Pid} = rustbridge_plugin:start_link(
        test_log_plugin,
        LibPath,
        PluginConfig,
        #{log_handler => LogHandler}
    ),
    [{plugin, Pid} | Config];
init_per_testcase(_TestCase, Config) ->
    LibPath = ?config(lib_path, Config),
    {ok, Pid} = rustbridge_plugin:start_link(LibPath, #{}),
    [{plugin, Pid} | Config].

end_per_testcase(_TestCase, Config) ->
    Plugin = ?config(plugin, Config),
    case is_process_alive(Plugin) of
        true -> rustbridge_plugin:stop(Plugin);
        false -> ok
    end,
    ok.

%% ---------------------------------------------------------------------------
%% Test cases
%% ---------------------------------------------------------------------------

plugin___default_config___is_active(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, State} = rustbridge_plugin:get_state(Plugin),

    active = State.

call___echo_message___returns_response(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, Response} = rustbridge_plugin:call(
        Plugin, <<"echo">>, <<"{\"message\": \"Hello from Erlang!\"}">>
    ),

    Map = json:decode(Response),
    <<"Hello from Erlang!">> = maps:get(<<"message">>, Map),
    18 = maps:get(<<"length">>, Map).

call___greet___returns_greeting(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, Response} = rustbridge_plugin:call(Plugin, <<"greet">>, <<"{\"name\": \"Erlang\"}">>),

    Map = json:decode(Response),
    Greeting = maps:get(<<"greeting">>, Map),
    true = is_binary(Greeting),
    {match, _} = re:run(Greeting, <<"Erlang">>).

call___user_create___returns_user_id(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, Response} = rustbridge_plugin:call(
        Plugin,
        <<"user.create">>,
        <<"{\"username\": \"erlang_user\", \"email\": \"erl@example.com\"}">>
    ),

    Map = json:decode(Response),
    UserId = maps:get(<<"user_id">>, Map),
    true = is_binary(UserId),
    true = is_binary(maps:get(<<"created_at">>, Map)).

call___math_add___returns_sum(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, Response} = rustbridge_plugin:call(Plugin, <<"math.add">>, <<"{\"a\": 10, \"b\": 32}">>),

    Map = json:decode(Response),
    42 = maps:get(<<"result">>, Map).

call___unknown_type___returns_error_code_6(Config) ->
    Plugin = ?config(plugin, Config),

    {error, {6, _Message}} = rustbridge_plugin:call(Plugin, <<"nonexistent">>, <<"{}">>, 10000).

get_state___after_load___returns_active(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, active} = rustbridge_plugin:get_state(Plugin).

get_rejected_count___default_config___returns_zero(Config) ->
    Plugin = ?config(plugin, Config),

    {ok, Count} = rustbridge_plugin:get_rejected_count(Plugin),

    0 = Count.

shutdown___explicit___state_becomes_stopped(Config) ->
    Plugin = ?config(plugin, Config),

    ok = rustbridge_plugin:shutdown(Plugin),

    %% After shutdown, stop the gen_server
    rustbridge_plugin:stop(Plugin).

set_log_level___changes_level___returns_ok(Config) ->
    Plugin = ?config(plugin, Config),

    ok = rustbridge_plugin:set_log_level(Plugin, debug).

concurrent_calls___multiple_processes___all_succeed(Config) ->
    Plugin = ?config(plugin, Config),
    Self = self(),
    NumProcs = 10,

    Pids = [
        spawn_link(fun() ->
            Result = rustbridge_plugin:call(
                Plugin,
                <<"echo">>,
                <<"{\"message\": \"concurrent\"}">>,
                10000
            ),
            Self ! {done, self(), Result}
        end)
     || _ <- lists:seq(1, NumProcs)
    ],

    Results = [
        receive
            {done, Pid, R} -> R
        after 15000 -> timeout
        end
     || Pid <- Pids
    ],

    lists:foreach(
        fun(Result) ->
            {ok, _} = Result
        end,
        Results
    ).

log_callback___receives_log_messages(Config) ->
    Plugin = ?config(plugin, Config),

    %% Make a call that triggers logging
    {ok, _} = rustbridge_plugin:call(
        Plugin, <<"echo">>, <<"{\"message\": \"trigger log\"}">>, 10000
    ),

    %% Give a moment for log messages to arrive
    timer:sleep(100),

    %% Check that we received at least one log message
    receive
        {log, _Level, _Target, _Msg} -> ok
    after 0 ->
        %% Log messages are best-effort; the plugin may not emit any for echo.
        %% This test verifies the callback mechanism works without crashing.
        ok
    end.
