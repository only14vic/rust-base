create table if not exists app.config (
    name text primary key,
    value text,
    default_value text,
    info text
);

comment on table app.config is 'Настройки приложения';

---------------

create or replace function app.config(
    name text,
    missing_ok bool default false,
    out value text
)
    returns text
    language plpgsql
    stable
    parallel safe
as $$
    # variable_conflict use_variable
begin
    value = current_setting(name, true);

    if value is null then
        select t.value
        into value
        from app.config as t
        where t.name = name;

        if not found and not missing_ok then
            perform app.raise('EXCEPTION',
                format('Invalid app config option name "%s".', name)
            );
        end if;
    end if;
end $$;

comment on function app.config(text, bool) is 'Получение настройки приложения';

------------

create or replace function app.config(
    name text,
    default_value text,
    out value text
)
    returns text
    language plpgsql
    stable
    parallel safe
as $$
    # variable_conflict use_variable
begin
    value = app.config(name, true);

    if value is null then
        value = default_value;
    end if;
end $$;

comment on function app.config(text, text) is 'Получение настройки или значения по умолчанию';
