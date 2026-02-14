-module(rustbridge_protocol_tests).
-include_lib("eunit/include/eunit.hrl").
-include("rustbridge.hrl").

%% ---------------------------------------------------------------------------
%% Encoding tests
%% ---------------------------------------------------------------------------

encode_load___basic___produces_valid_json_test() ->
    Bin = rustbridge_protocol:encode_load(1, "/tmp/lib.so", #{}),

    Map = json:decode(Bin),

    ?assertEqual(<<"load">>, maps:get(<<"type">>, Map)),
    ?assertEqual(1, maps:get(<<"id">>, Map)),
    ?assertEqual(<<"/tmp/lib.so">>, maps:get(<<"path">>, Map)).

encode_load___with_config___includes_config_test() ->
    Config = #{<<"log_level">> => <<"debug">>},

    Bin = rustbridge_protocol:encode_load(2, "/tmp/lib.so", Config),

    Map = json:decode(Bin),
    ConfigMap = maps:get(<<"config">>, Map),
    ?assertEqual(<<"debug">>, maps:get(<<"log_level">>, ConfigMap)).

encode_call___basic___produces_valid_json_test() ->
    Bin = rustbridge_protocol:encode_call(3, <<"echo">>, <<"{\"message\": \"hello\"}">>),

    Map = json:decode(Bin),

    ?assertEqual(<<"call">>, maps:get(<<"type">>, Map)),
    ?assertEqual(3, maps:get(<<"id">>, Map)),
    ?assertEqual(<<"echo">>, maps:get(<<"type_tag">>, Map)),
    ?assertEqual(<<"{\"message\": \"hello\"}">>, maps:get(<<"payload">>, Map)).

encode_call___string_type_tag___converts_to_binary_test() ->
    Bin = rustbridge_protocol:encode_call(4, "greet", <<"test">>),

    Map = json:decode(Bin),

    ?assertEqual(<<"greet">>, maps:get(<<"type_tag">>, Map)).

encode_call_raw___basic___encodes_data_as_base64_test() ->
    Data = <<1, 2, 3>>,

    Bin = rustbridge_protocol:encode_call_raw(5, 1, Data),

    Map = json:decode(Bin),
    ?assertEqual(<<"call_raw">>, maps:get(<<"type">>, Map)),
    ?assertEqual(5, maps:get(<<"id">>, Map)),
    ?assertEqual(1, maps:get(<<"message_id">>, Map)),
    B64 = maps:get(<<"data">>, Map),
    ?assertEqual(Data, base64:decode(B64)).

encode_get_state___basic___produces_valid_json_test() ->
    Bin = rustbridge_protocol:encode_get_state(6),

    Map = json:decode(Bin),

    ?assertEqual(<<"get_state">>, maps:get(<<"type">>, Map)),
    ?assertEqual(6, maps:get(<<"id">>, Map)).

encode_set_log_level___basic___produces_valid_json_test() ->
    Bin = rustbridge_protocol:encode_set_log_level(7, 3),

    Map = json:decode(Bin),

    ?assertEqual(<<"set_log_level">>, maps:get(<<"type">>, Map)),
    ?assertEqual(7, maps:get(<<"id">>, Map)),
    ?assertEqual(3, maps:get(<<"level">>, Map)).

encode_get_rejected_count___basic___produces_valid_json_test() ->
    Bin = rustbridge_protocol:encode_get_rejected_count(10),

    Map = json:decode(Bin),

    ?assertEqual(<<"get_rejected_count">>, maps:get(<<"type">>, Map)),
    ?assertEqual(10, maps:get(<<"id">>, Map)).

encode_shutdown___basic___produces_valid_json_test() ->
    Bin = rustbridge_protocol:encode_shutdown(8),

    Map = json:decode(Bin),

    ?assertEqual(<<"shutdown">>, maps:get(<<"type">>, Map)),
    ?assertEqual(8, maps:get(<<"id">>, Map)).

encode_load_bundle___with_verify___produces_valid_json_test() ->
    Opts = #{verify_signatures => true, public_key => <<"abc123">>},

    Bin = rustbridge_protocol:encode_load_bundle(9, "/tmp/plugin.rbp", #{}, Opts),

    Map = json:decode(Bin),
    ?assertEqual(<<"load_bundle">>, maps:get(<<"type">>, Map)),
    ?assertEqual(true, maps:get(<<"verify_signatures">>, Map)),
    ?assertEqual(<<"abc123">>, maps:get(<<"public_key">>, Map)).

%% ---------------------------------------------------------------------------
%% Decoding tests
%% ---------------------------------------------------------------------------

decode_message___ok_response___returns_ok_tuple_test() ->
    Json = iolist_to_binary(
        json:encode(#{
            <<"type">> => <<"response">>,
            <<"id">> => 1,
            <<"status">> => <<"ok">>,
            <<"data">> => <<"active">>
        })
    ),

    Result = rustbridge_protocol:decode_message(Json),

    ?assertEqual({response, 1, {ok, <<"active">>}}, Result).

decode_message___error_response___returns_error_tuple_test() ->
    Json = iolist_to_binary(
        json:encode(#{
            <<"type">> => <<"response">>,
            <<"id">> => 2,
            <<"status">> => <<"error">>,
            <<"error_code">> => 6,
            <<"error_message">> => <<"unknown message type">>
        })
    ),

    Result = rustbridge_protocol:decode_message(Json),

    ?assertEqual({response, 2, {error, {6, <<"unknown message type">>}}}, Result).

decode_message___log_message___returns_log_entry_test() ->
    Json = iolist_to_binary(
        json:encode(#{
            <<"type">> => <<"log">>,
            <<"level">> => 2,
            <<"target">> => <<"hello_plugin">>,
            <<"message">> => <<"starting up">>
        })
    ),

    {log, Entry} = rustbridge_protocol:decode_message(Json),

    ?assertEqual(info, Entry#log_entry.level),
    ?assertEqual(<<"hello_plugin">>, Entry#log_entry.target),
    ?assertEqual(<<"starting up">>, Entry#log_entry.message).

decode_message___ok_response_with_null_data___returns_null_test() ->
    Json = iolist_to_binary(
        json:encode(#{
            <<"type">> => <<"response">>,
            <<"id">> => 3,
            <<"status">> => <<"ok">>
        })
    ),

    Result = rustbridge_protocol:decode_message(Json),

    ?assertEqual({response, 3, {ok, null}}, Result).
