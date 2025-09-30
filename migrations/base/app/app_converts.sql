create or replace function app.to_text(anyelement)
    returns text
    language sql
    immutable
    parallel safe
as $$
    select $1::text;
$$;

comment on function app.to_text(anyelement) is 'Перевести тип в текст';

--------------

create or replace function app.to_text(anyarray, delimiter text)
    returns text
    language sql
    immutable
    parallel safe
as $$
    select array_to_string($1, delimiter);
$$;

comment on function app.to_text(anyarray, text) is 'Перевести массив в текст с разделителем';

--------------

create or replace function app.to_date(anyelement)
    returns date
    language sql
    immutable
    parallel safe
as $$
    select $1::date;
$$;

comment on function app.to_date(anyelement) is 'Перевести тип в дату';

--------------
