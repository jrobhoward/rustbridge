-module(rustbridge_log_tests).
-include_lib("eunit/include/eunit.hrl").

to_code___trace___returns_0_test() ->
    ?assertEqual(0, rustbridge_log:to_code(trace)).

to_code___debug___returns_1_test() ->
    ?assertEqual(1, rustbridge_log:to_code(debug)).

to_code___info___returns_2_test() ->
    ?assertEqual(2, rustbridge_log:to_code(info)).

to_code___warn___returns_3_test() ->
    ?assertEqual(3, rustbridge_log:to_code(warn)).

to_code___error___returns_4_test() ->
    ?assertEqual(4, rustbridge_log:to_code(error)).

to_code___off___returns_5_test() ->
    ?assertEqual(5, rustbridge_log:to_code(off)).

from_code___valid_codes___returns_correct_levels_test() ->
    ?assertEqual(trace, rustbridge_log:from_code(0)),
    ?assertEqual(debug, rustbridge_log:from_code(1)),
    ?assertEqual(info, rustbridge_log:from_code(2)),
    ?assertEqual(warn, rustbridge_log:from_code(3)),
    ?assertEqual(error, rustbridge_log:from_code(4)).

from_code___unknown_code___returns_off_test() ->
    ?assertEqual(off, rustbridge_log:from_code(99)),
    ?assertEqual(off, rustbridge_log:from_code(255)).

to_logger_level___trace___maps_to_debug_test() ->
    ?assertEqual(debug, rustbridge_log:to_logger_level(trace)).

to_logger_level___debug___maps_to_debug_test() ->
    ?assertEqual(debug, rustbridge_log:to_logger_level(debug)).

to_logger_level___info___maps_to_info_test() ->
    ?assertEqual(info, rustbridge_log:to_logger_level(info)).

to_logger_level___warn___maps_to_warning_test() ->
    ?assertEqual(warning, rustbridge_log:to_logger_level(warn)).

to_logger_level___error___maps_to_error_test() ->
    ?assertEqual(error, rustbridge_log:to_logger_level(error)).

to_logger_level___off___returns_error_test() ->
    ?assertEqual(error, rustbridge_log:to_logger_level(off)).
