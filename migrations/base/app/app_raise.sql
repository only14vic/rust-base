create type app.raise_level as enum (
    'FORBIDDEN',
    'UNAUTHORIZED',
    'INVALID',
    'ASSERT',
    'EXCEPTION',
    'ERROR',
    'WARNING',
    'NOTICE',
    'DEBUG',
    'LOG',
    'INFO'
);

comment on type app.raise_level is 'Уровень ошибки для app.raise';

create or replace function app.raise(
    level app.raise_level,
    message text,
    details text = null,
    hint text = null
)
    returns void
    language plpgsql
as $$
begin
    message := coalesce(message, '');
    details := coalesce(details, '');
    hint := coalesce(hint, '');

    case level
        when 'FORBIDDEN'
            then raise insufficient_privilege using message = message, detail = details, hint = hint;
        when 'UNAUTHORIZED'
            then raise sqlstate 'PT401' using message = message, detail = details, hint = hint;
        when 'INVALID'
            then raise data_exception using message = message, detail = details, hint = hint;
        when 'ASSERT'
            then raise assert_failure using message = message, detail = details, hint = hint;
        when 'EXCEPTION'
            then raise internal_error using message = message, detail = details, hint = hint;
        when 'ERROR'
            then raise exception '%', message using detail = details, hint = hint;
        when 'WARNING'
            then raise warning '%', message using detail = details, hint = hint;
        when 'NOTICE'
            then raise notice '%', message using detail = details, hint = hint;
        when 'DEBUG'
            then raise debug '%', message using detail = details, hint = hint;
        when 'LOG'
            then raise log '%', message using detail = details, hint = hint;
        when 'INFO'
            then raise info '%', message using detail = details, hint = hint;
        else
            raise exception 'Unsupported raise level: "%"', level::text;
    end case;
end $$;

comment on function app.raise is 'Выброс ошибки';

----------------
