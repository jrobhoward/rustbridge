-module(rustbridge_protocol).

-include("rustbridge.hrl").

-export([
    encode_load/3,
    encode_load_bundle/4,
    encode_call/3,
    encode_call_raw/3,
    encode_get_state/1,
    encode_get_rejected_count/1,
    encode_set_log_level/2,
    encode_shutdown/1,
    decode_message/1
]).

%% ---------------------------------------------------------------------------
%% Encoding commands (Erlang -> port driver)
%% ---------------------------------------------------------------------------

-spec encode_load(non_neg_integer(), string() | binary(), map()) -> binary().
encode_load(Id, Path, ConfigMap) ->
    Cmd = #{
        <<"type">> => <<"load">>,
        <<"id">> => Id,
        <<"path">> => to_binary(Path),
        <<"config">> => ConfigMap
    },
    iolist_to_binary(json:encode(Cmd)).

-spec encode_load_bundle(non_neg_integer(), string() | binary(), map(), map()) -> binary().
encode_load_bundle(Id, Path, ConfigMap, Opts) ->
    Cmd0 = #{
        <<"type">> => <<"load_bundle">>,
        <<"id">> => Id,
        <<"path">> => to_binary(Path),
        <<"config">> => ConfigMap
    },
    Cmd1 =
        case maps:get(verify_signatures, Opts, undefined) of
            undefined -> Cmd0;
            V -> Cmd0#{<<"verify_signatures">> => V}
        end,
    Cmd2 =
        case maps:get(public_key, Opts, undefined) of
            undefined -> Cmd1;
            K -> Cmd1#{<<"public_key">> => to_binary(K)}
        end,
    iolist_to_binary(json:encode(Cmd2)).

-spec encode_call(non_neg_integer(), type_tag(), binary()) -> binary().
encode_call(Id, TypeTag, Payload) ->
    Cmd = #{
        <<"type">> => <<"call">>,
        <<"id">> => Id,
        <<"type_tag">> => to_binary(TypeTag),
        <<"payload">> => Payload
    },
    iolist_to_binary(json:encode(Cmd)).

-spec encode_call_raw(non_neg_integer(), non_neg_integer(), binary()) -> binary().
encode_call_raw(Id, MessageId, Data) ->
    Cmd = #{
        <<"type">> => <<"call_raw">>,
        <<"id">> => Id,
        <<"message_id">> => MessageId,
        <<"data">> => base64:encode(Data)
    },
    iolist_to_binary(json:encode(Cmd)).

-spec encode_get_state(non_neg_integer()) -> binary().
encode_get_state(Id) ->
    Cmd = #{
        <<"type">> => <<"get_state">>,
        <<"id">> => Id
    },
    iolist_to_binary(json:encode(Cmd)).

-spec encode_get_rejected_count(non_neg_integer()) -> binary().
encode_get_rejected_count(Id) ->
    Cmd = #{
        <<"type">> => <<"get_rejected_count">>,
        <<"id">> => Id
    },
    iolist_to_binary(json:encode(Cmd)).

-spec encode_set_log_level(non_neg_integer(), non_neg_integer()) -> binary().
encode_set_log_level(Id, LevelCode) ->
    Cmd = #{
        <<"type">> => <<"set_log_level">>,
        <<"id">> => Id,
        <<"level">> => LevelCode
    },
    iolist_to_binary(json:encode(Cmd)).

-spec encode_shutdown(non_neg_integer()) -> binary().
encode_shutdown(Id) ->
    Cmd = #{
        <<"type">> => <<"shutdown">>,
        <<"id">> => Id
    },
    iolist_to_binary(json:encode(Cmd)).

%% ---------------------------------------------------------------------------
%% Decoding messages (port driver -> Erlang)
%% ---------------------------------------------------------------------------

-spec decode_message(binary()) ->
    {response, non_neg_integer(), {ok, term()} | {error, {integer(), binary()}}}
    | {log, #log_entry{}}.
decode_message(JsonBin) ->
    Map = json:decode(JsonBin),
    case maps:get(<<"type">>, Map) of
        <<"response">> ->
            decode_response(Map);
        <<"log">> ->
            decode_log(Map)
    end.

%% ---------------------------------------------------------------------------
%% Internal
%% ---------------------------------------------------------------------------

decode_response(Map) ->
    Id = maps:get(<<"id">>, Map),
    case maps:get(<<"status">>, Map) of
        <<"ok">> ->
            Data = maps:get(<<"data">>, Map, null),
            {response, Id, {ok, Data}};
        <<"error">> ->
            Code = maps:get(<<"error_code">>, Map, 11),
            Msg = maps:get(<<"error_message">>, Map, <<"">>),
            {response, Id, {error, {Code, Msg}}}
    end.

decode_log(Map) ->
    Level = rustbridge_log:from_code(maps:get(<<"level">>, Map)),
    Target = maps:get(<<"target">>, Map, <<"">>),
    Message = maps:get(<<"message">>, Map, <<"">>),
    {log, #log_entry{level = Level, target = Target, message = Message}}.

to_binary(V) when is_binary(V) -> V;
to_binary(V) when is_list(V) -> list_to_binary(V);
to_binary(V) when is_atom(V) -> atom_to_binary(V, utf8).
