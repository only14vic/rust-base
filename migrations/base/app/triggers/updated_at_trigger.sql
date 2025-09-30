create or replace function app.updated_at_trigger()
    returns trigger
    language plpgsql
    stable
as $$
begin
    -- @migration-sort-ignore-target: auth.current_user_is_usesuper

    if TG_ARGV[0] = 'check'
        and TG_OP = 'UPDATE'
        and not auth.current_user_is_usesuper()
        and OLD.updated_at is distinct from NEW.updated_at
    then
        perform app.raise(
            'INVALID',
            'Data was previously modified.',
            format('%s %s', TG_OP, TG_TABLE_NAME)
        );
    end if;

    NEW.updated_at = now();

    return NEW;
end $$;

comment on function app.updated_at_trigger is 'Устанавливает текущую дату в поле updated_at';
