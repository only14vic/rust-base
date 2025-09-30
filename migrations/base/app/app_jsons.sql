create or replace function app.jsonb_diff(jsonb, jsonb)
    returns jsonb
    language sql
    stable
as $$
    select jsonb_object_agg(key, value)
    from ( select *
           from jsonb_each($2)
           except
           select *
           from jsonb_each($1) ) as t;
$$;

comment on function app.jsonb_diff is 'Разница полей json объектов';

-----------

create or replace function app.record_diff_keys(anyelement, anyelement)
    returns text[]
    language sql
    stable
as $$
    select coalesce(array_agg(k), '{}'::text[])
    from jsonb_object_keys(
            jsonb_diff(
                to_jsonb($1),
                to_jsonb($2)
            )
        ) as t(k),
        (values(1)) as tt;
$$;

comment on function app.record_diff_keys is 'Спсисок полей отличающихся у двух записей';

-----------

create or replace function app.jsonb_merge(a jsonb, b jsonb)
    returns jsonb
    language sql
    stable
as $$
    select
        jsonb_object_agg(
            coalesce(ka, kb),
            case
                when va is null then vb
                when vb is null then va
                when va = vb then va
                else va || vb
            end
        )
    from jsonb_each(a) e1(ka, va)
    full join jsonb_each(b) e2(kb, vb) on ka = kb
$$;

comment on function app.jsonb_merge is 'Соединение двух jsonb';

------------
