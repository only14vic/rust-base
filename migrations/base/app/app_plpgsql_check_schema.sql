create or replace function app.plpgsql_check_schema(schema text[])
    returns void
    language sql
as $$
    with t as (
        select p.oid, n.nspname, p.proname, tgrelid::regclass, ty.typname
        from pg_proc p
        left join pg_trigger t on t.tgfoid = p.oid
        join pg_type ty on p.prorettype = ty.oid
        join pg_language l on p.prolang = l.oid
        join pg_namespace n on p.pronamespace = n.oid
        where n.nspname = any(schema) and l.lanname = 'plpgsql'
        order by n.nspname, p.proname
    ),
    tt as (
        select format('%s (for %s)',proname,tgrelid) as proname, nspname, array_to_string(array_agg(cf), E'\n') as err
        from t,
        lateral public.plpgsql_check_function(oid, tgrelid) as cf
        where tgrelid is not null
        group by proname, nspname, tgrelid
        union
        select proname, nspname, array_to_string(array_agg(cf), E'\n') as err
        from t,
        lateral public.plpgsql_check_function(oid) as cf
        where typname != 'trigger'
        group by proname, nspname
    )
    select app.raise(
        'EXCEPTION',
        E'plpgsql_check errors:\n' || array_to_string(
            array_agg(format('%s.%s: %s', nspname, proname, err)),
            E'\n'
        )
    )
    from tt
    group by nspname, proname;
$$;

comment on function app.plpgsql_check_schema is 'Проверка кода plpgsql';
