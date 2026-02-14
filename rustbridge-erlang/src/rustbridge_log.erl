-module(rustbridge_log).

-export([to_code/1, from_code/1, to_logger_level/1]).

-type level() :: trace | debug | info | warn | error | off.
-export_type([level/0]).

%% @doc Convert a log level atom to FFI code.
-spec to_code(level()) -> 0..5.
to_code(trace) -> 0;
to_code(debug) -> 1;
to_code(info) -> 2;
to_code(warn) -> 3;
to_code(error) -> 4;
to_code(off) -> 5.

%% @doc Convert an FFI code to a log level atom.
-spec from_code(non_neg_integer()) -> level().
from_code(0) -> trace;
from_code(1) -> debug;
from_code(2) -> info;
from_code(3) -> warn;
from_code(4) -> error;
from_code(_) -> off.

%% @doc Convert a rustbridge log level to an OTP logger level.
%% OTP logger has no 'trace' level, so trace maps to debug.
%% OTP uses 'warning' instead of 'warn'.
-spec to_logger_level(level()) -> logger:level().
to_logger_level(trace) -> debug;
to_logger_level(debug) -> debug;
to_logger_level(info) -> info;
to_logger_level(warn) -> warning;
to_logger_level(error) -> error;
to_logger_level(off) -> error.
