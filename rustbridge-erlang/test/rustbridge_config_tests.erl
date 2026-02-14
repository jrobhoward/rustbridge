-module(rustbridge_config_tests).
-include_lib("eunit/include/eunit.hrl").
-include("rustbridge.hrl").

defaults___returns_default_values_test() ->
    Config = rustbridge_config:defaults(),

    ?assertEqual(info, Config#plugin_config.log_level),
    ?assertEqual(1000, Config#plugin_config.max_concurrent_ops),
    ?assertEqual(5000, Config#plugin_config.shutdown_timeout_ms),
    ?assertEqual(#{}, Config#plugin_config.data),
    ?assertEqual(undefined, Config#plugin_config.worker_threads),
    ?assertEqual(undefined, Config#plugin_config.init_params).

to_json_map___defaults___has_required_fields_test() ->
    Config = rustbridge_config:defaults(),

    Map = rustbridge_config:to_json_map(Config),

    ?assertEqual(<<"info">>, maps:get(<<"log_level">>, Map)),
    ?assertEqual(1000, maps:get(<<"max_concurrent_ops">>, Map)),
    ?assertEqual(5000, maps:get(<<"shutdown_timeout_ms">>, Map)),
    ?assertEqual(#{}, maps:get(<<"data">>, Map)),
    ?assertEqual(false, maps:is_key(<<"worker_threads">>, Map)),
    ?assertEqual(false, maps:is_key(<<"init_params">>, Map)).

to_json_map___with_worker_threads___includes_field_test() ->
    Config = #plugin_config{worker_threads = 4},

    Map = rustbridge_config:to_json_map(Config),

    ?assertEqual(4, maps:get(<<"worker_threads">>, Map)).

to_json_map___with_init_params___includes_field_test() ->
    Config = #plugin_config{init_params = #{<<"key">> => <<"value">>}},

    Map = rustbridge_config:to_json_map(Config),

    ?assertEqual(#{<<"key">> => <<"value">>}, maps:get(<<"init_params">>, Map)).

to_json_map___custom_log_level___serializes_correctly_test() ->
    Config = #plugin_config{log_level = debug},

    Map = rustbridge_config:to_json_map(Config),

    ?assertEqual(<<"debug">>, maps:get(<<"log_level">>, Map)).
