create or replace function app.deleted_at_trigger()
    returns trigger
    language plpgsql
    security definer
as $$
declare
    table_name text;
begin
    if tg_op != 'DELETE' then
        raise exception 'The "deleted_at_trigger" required DELETE operation.';
    end if;

    if old.deleted_at is not null then
        return old;
    end if;

    table_name = quote_ident(tg_table_schema) || '.' || quote_ident(tg_table_name);

    execute format(
        'update %s
         set deleted_at = now()
         where row(%s) = row(%s)',
         table_name,
         array_to_string(app.table_primary_key(table_name), ','),
         '$1.' || array_to_string(app.table_primary_key(table_name), ',$1.')
    ) using old;

    return null;
end $$;

comment on function app.deleted_at_trigger is 'Установка даты удаления вместо реального удаления записи';
