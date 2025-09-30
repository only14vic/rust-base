create or replace function app.forbidden_trigger()
    returns trigger
    language plpgsql
as $$
begin
    select app.raise('FORBIDDEN', coalesce(tg_argv[0]::text, 'Fobidden'), tg_argv[1]::text);
end $$;

comment on function app.forbidden_trigger is 'Запрет действия';
