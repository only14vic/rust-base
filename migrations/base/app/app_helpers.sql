create or replace function app.current_lang()
    returns text
    language sql
    stable
    parallel safe
as $$
    select substring(current_setting('lc_messages') for 2);
$$;

comment on function app.current_lang is 'Текущий язык локали';

----------------

create or replace function app.table_name(table_name text)
    returns text
    language sql
    immutable
    parallel safe
as $$
    select
        case
           when position('.' in table_name) > 0
               then substring(table_name from position('.' in table_name) + 1)
           else table_name
        end;
$$;

----------------

create or replace function app.schema_name(table_name text)
    returns text
    language sql
    immutable
    parallel safe
as $$
    select
        case
           when position('.' in table_name) > 0
               then substring(table_name for position('.' in table_name) - 1)
           else current_schema()
       end;
$$;

----------------

create or replace function app.table_primary_key(table_name text)
    returns text[]
    language sql
    immutable
    parallel safe
    security definer
as $$
    select array_agg(a.attname)
    from pg_index i
    join pg_attribute a
        on a.attrelid = i.indrelid
        and a.attnum = any (i.indkey)
    where i.indrelid = table_name::regclass
      and i.indisprimary;
$$;

----------------
