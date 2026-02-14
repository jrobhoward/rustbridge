-module(rustbridge_bench_SUITE).
-include_lib("common_test/include/ct.hrl").
-include("rustbridge.hrl").

%% CT callbacks
-export([all/0, groups/0, init_per_suite/1, end_per_suite/1,
         init_per_group/2, end_per_group/2,
         init_per_testcase/2, end_per_testcase/2]).

%% Test cases
-export([
    json_echo___small_payload___latency/1,
    json_echo___medium_payload___latency/1,
    binary___small_request___latency/1,
    concurrent___json_echo___throughput/1
]).

-define(WARMUP_ITERATIONS, 100).
-define(BENCH_ITERATIONS, 5000).
-define(CONCURRENT_PROCS, 10).
-define(CONCURRENT_CALLS_PER_PROC, 500).

%% ---------------------------------------------------------------------------
%% CT callbacks
%% ---------------------------------------------------------------------------

all() ->
    [{group, benchmarks}].

groups() ->
    [{benchmarks, [sequence], [
        json_echo___small_payload___latency,
        json_echo___medium_payload___latency,
        binary___small_request___latency,
        concurrent___json_echo___throughput
    ]}].

init_per_suite(Config) ->
    PrivDir = code:priv_dir(rustbridge),
    RebarProjectRoot = find_rebar_root(PrivDir),
    WorkspaceRoot = filename:dirname(RebarProjectRoot),

    LibName = case os:type() of
        {unix, darwin} -> "libhello_plugin.dylib";
        {unix, _}      -> "libhello_plugin.so";
        {win32, _}     -> "hello_plugin.dll"
    end,
    LibPath = filename:join([WorkspaceRoot, "target", "release", LibName]),

    case filelib:is_file(LibPath) of
        true ->
            [{lib_path, LibPath} | Config];
        false ->
            {skip, {hello_plugin_not_found, LibPath}}
    end.

find_rebar_root(Dir) ->
    case filelib:is_file(filename:join(Dir, "rebar.config")) of
        true -> Dir;
        false ->
            Parent = filename:dirname(Dir),
            case Parent of
                Dir -> error({rebar_config_not_found, Dir});
                _   -> find_rebar_root(Parent)
            end
    end.

end_per_suite(_Config) ->
    ok.

init_per_group(_Group, Config) ->
    Config.

end_per_group(_Group, _Config) ->
    ok.

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
%% Benchmarks
%% ---------------------------------------------------------------------------

json_echo___small_payload___latency(Config) ->
    Plugin = ?config(plugin, Config),
    Payload = <<"{\"message\": \"benchmark test\"}">>,

    %% Warmup
    lists:foreach(fun(_) ->
        {ok, _} = rustbridge_plugin:call(Plugin, <<"echo">>, Payload, 10000)
    end, lists:seq(1, ?WARMUP_ITERATIONS)),

    %% Measure
    Times = lists:map(fun(_) ->
        {T, {ok, _}} = timer:tc(fun() ->
            rustbridge_plugin:call(Plugin, <<"echo">>, Payload, 10000)
        end),
        T
    end, lists:seq(1, ?BENCH_ITERATIONS)),

    report("JSON echo (small, 30 B)", Times).

json_echo___medium_payload___latency(Config) ->
    Plugin = ?config(plugin, Config),
    MsgBody = list_to_binary(lists:duplicate(900, $x)),
    Payload = iolist_to_binary([<<"{\"message\": \"">>, MsgBody, <<"\"}">>]),

    %% Warmup
    lists:foreach(fun(_) ->
        {ok, _} = rustbridge_plugin:call(Plugin, <<"echo">>, Payload, 10000)
    end, lists:seq(1, ?WARMUP_ITERATIONS)),

    %% Measure
    Times = lists:map(fun(_) ->
        {T, {ok, _}} = timer:tc(fun() ->
            rustbridge_plugin:call(Plugin, <<"echo">>, Payload, 10000)
        end),
        T
    end, lists:seq(1, ?BENCH_ITERATIONS)),

    report("JSON echo (medium, 1 KB)", Times).

binary___small_request___latency(Config) ->
    Plugin = ?config(plugin, Config),
    Request = build_small_binary_request(<<"bench_key">>, 16#01),

    %% Warmup
    lists:foreach(fun(_) ->
        {ok, _} = rustbridge_plugin:call_raw(Plugin, 1, Request, 10000)
    end, lists:seq(1, ?WARMUP_ITERATIONS)),

    %% Measure
    Times = lists:map(fun(_) ->
        {T, {ok, _}} = timer:tc(fun() ->
            rustbridge_plugin:call_raw(Plugin, 1, Request, 10000)
        end),
        T
    end, lists:seq(1, ?BENCH_ITERATIONS)),

    report("Binary (small, 76 B request)", Times).

concurrent___json_echo___throughput(Config) ->
    Plugin = ?config(plugin, Config),
    Payload = <<"{\"message\": \"concurrent test\"}">>,
    CallsPerProc = ?CONCURRENT_CALLS_PER_PROC,
    NumProcs = ?CONCURRENT_PROCS,
    Self = self(),

    %% Warmup with a few sequential calls
    lists:foreach(fun(_) ->
        {ok, _} = rustbridge_plugin:call(Plugin, <<"echo">>, Payload, 10000)
    end, lists:seq(1, ?WARMUP_ITERATIONS)),

    %% Measure: spawn NumProcs processes, each making CallsPerProc calls
    T0 = erlang:monotonic_time(microsecond),
    Pids = [spawn_link(fun() ->
        lists:foreach(fun(_) ->
            {ok, _} = rustbridge_plugin:call(Plugin, <<"echo">>, Payload, 30000)
        end, lists:seq(1, CallsPerProc)),
        Self ! {done, self()}
    end) || _ <- lists:seq(1, NumProcs)],

    lists:foreach(fun(Pid) ->
        receive {done, Pid} -> ok after 60000 -> error(timeout) end
    end, Pids),
    T1 = erlang:monotonic_time(microsecond),

    TotalCalls = NumProcs * CallsPerProc,
    ElapsedUs = T1 - T0,
    ElapsedMs = ElapsedUs / 1000.0,
    OpsPerSec = TotalCalls / (ElapsedUs / 1_000_000.0),
    MeanUs = ElapsedUs / TotalCalls,

    Output = lists:flatten(io_lib:format(
        "~n=== Concurrent JSON echo (~B procs x ~B calls) ===~n"
        "  Total calls:   ~B~n"
        "  Elapsed:       ~s ms~n"
        "  Mean latency:  ~s~n"
        "  Throughput:    ~s ops/s~n",
        [NumProcs, CallsPerProc, TotalCalls,
         lists:flatten(io_lib:format("~.1f", [ElapsedMs * 1.0])),
         format_time(MeanUs), format_ops(OpsPerSec)])),
    ct:pal("~ts", [Output]).

%% ---------------------------------------------------------------------------
%% Binary request builder
%% ---------------------------------------------------------------------------

%% Builds a 76-byte SmallRequestRaw struct (repr(C), native/little-endian):
%%   version:   u8       = 1
%%   _reserved: [u8; 3]  = 0
%%   key:       [u8; 64] = key padded with zeros
%%   key_len:   u32      = byte_size(Key)
%%   flags:     u32      = Flags
build_small_binary_request(Key, Flags) when byte_size(Key) =< 64 ->
    PadLen = 64 - byte_size(Key),
    <<1:8, 0:24, Key/binary, 0:(PadLen * 8),
      (byte_size(Key)):32/native, Flags:32/native>>.

%% ---------------------------------------------------------------------------
%% Statistics and reporting
%% ---------------------------------------------------------------------------

report(Label, Times) ->
    Sorted = lists:sort(Times),
    N = length(Sorted),
    Min = hd(Sorted),
    Max = lists:last(Sorted),
    Mean = lists:sum(Sorted) / N,
    Median = percentile(Sorted, 50),
    P99 = percentile(Sorted, 99),
    StdDev = stddev(Sorted, Mean),
    OpsPerSec = 1_000_000.0 / Mean,

    Output = lists:flatten(io_lib:format(
        "~n=== ~s (~B iterations) ===~n"
        "  Min:           ~s~n"
        "  Max:           ~s~n"
        "  Mean:          ~s~n"
        "  Median:        ~s~n"
        "  P99:           ~s~n"
        "  Std Dev:       ~s~n"
        "  Throughput:    ~s ops/s~n",
        [Label, N,
         format_time(Min), format_time(Max), format_time(Mean),
         format_time(Median), format_time(P99), format_time(StdDev),
         format_ops(OpsPerSec)])),
    ct:pal("~ts", [Output]).

percentile(Sorted, P) ->
    N = length(Sorted),
    Rank = (P / 100.0) * (N - 1) + 1,
    Lower = max(1, trunc(Rank)),
    Upper = min(N, Lower + 1),
    Frac = Rank - trunc(Rank),
    lists:nth(Lower, Sorted) * (1 - Frac) + lists:nth(Upper, Sorted) * Frac.

stddev(Sorted, Mean) ->
    N = length(Sorted),
    SumSq = lists:foldl(fun(X, Acc) -> Acc + (X - Mean) * (X - Mean) end, 0.0, Sorted),
    math:sqrt(SumSq / N).

format_time(Us) when Us >= 1000 ->
    lists:flatten(io_lib:format("~.2f ms", [Us / 1000.0]));
format_time(Us) when Us >= 1 ->
    lists:flatten(io_lib:format("~.1f us", [Us * 1.0]));
format_time(Us) ->
    integer_to_list(round(Us * 1000)) ++ " ns".

format_ops(Ops) ->
    integer_to_list(round(Ops)).
