-module(rustbridge_sup).
-behaviour(supervisor).

-export([start_link/0, init/1]).
-export([start_plugin/3, start_plugin/4, stop_plugin/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    SupFlags = #{strategy => one_for_one, intensity => 5, period => 60},
    {ok, {SupFlags, []}}.

%% @doc Start a named plugin under the supervisor.
-spec start_plugin(atom(), string(), #{} | tuple()) -> {ok, pid()} | {error, term()}.
start_plugin(Name, Path, Config) ->
    start_plugin(Name, Path, Config, #{}).

%% @doc Start a named plugin under the supervisor with options.
-spec start_plugin(atom(), string(), #{} | tuple(), map()) -> {ok, pid()} | {error, term()}.
start_plugin(Name, Path, Config, Opts) ->
    ChildSpec = #{
        id => Name,
        start => {rustbridge_plugin, start_link, [Name, Path, Config, Opts]},
        restart => transient,
        shutdown => 10000,
        type => worker
    },
    supervisor:start_child(?MODULE, ChildSpec).

%% @doc Stop a named plugin.
-spec stop_plugin(atom()) -> ok | {error, term()}.
stop_plugin(Name) ->
    case supervisor:terminate_child(?MODULE, Name) of
        ok -> supervisor:delete_child(?MODULE, Name);
        Error -> Error
    end.
