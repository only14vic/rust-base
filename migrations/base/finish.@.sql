do $$
begin
    if current_setting('plpgsql.check_asserts') != 'on' then
        raise exception 'Option "plpgsql.check_asserts" must be turn on.';
    end if;

    perform app.plpgsql_check_schema(array['app','api','auth']);

    perform postgrest.reload_config();
end $$;
