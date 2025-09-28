create or replace function api.debug_request()
    returns json
    language sql
    stable
    security definer
as $$
    select json_build_object(
       'request.jwt.claims', current_setting('request.jwt.claims', true)::json,
       'request.headers', current_setting('request.headers', true)::json,
       'request.cookies', current_setting('request.cookies', true)::json,
       'request.path', current_setting('request.path', true),
       'request.method', current_setting('request.method', true),
       'response.headers', current_setting('response.headers', true),
       'time', now(),
       --'user_id', auth.current_user_id(),
       'role', current_setting('role', true),
       'lang', app.current_lang(),
       --'user', ( select to_json(u) from app.users as u where u.id = auth.current_user_id() ),
       'locale', ( select json_agg(json_build_object(name, setting)) from pg_settings where name like 'lc_%' ),
       'enable', ( select json_agg(json_build_object(name, setting)) from pg_settings where name like 'enable_%' ),
       'settings', ( select json_agg(json_build_object(name, setting, 'unit', unit, 'source', source))
                        from pg_settings where name ~ 'statement_timeout|^work_mem' )
    );
$$;

comment on function api.debug_request is 'Отладка запроса';

grant execute on function api.debug_request to api;

--revoke all on function api.debug_request from public;
