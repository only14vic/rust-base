create or replace function postgrest.reload_config()
    returns void
    language sql
as $$
    notify pgrst, 'reload config';
$$;

comment on function postgrest.reload_config is 'Обновить кэш схемы базы в postgrest';

create or replace function postgrest.pre_config()
    returns void
    language plpgsql
    stable
as $$
begin

end $$;

comment on function postgrest.pre_config is 'Преднастройка postgrest';

create or replace function postgrest.pre_request()
    returns void
    language plpgsql
    security definer
    stable
    parallel safe
as $$
begin
    --perform app.set_locale(app.request_lang());
    --perform auth.check_current_token();
end $$;

comment on function postgrest.pre_request is 'Настройки перед выполнением запроса postgrest';

------------
