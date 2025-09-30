create or replace function app.queue_trigger()
    returns trigger
    language plpgsql
    security definer
as $$
    # variable_conflict use_column
declare
    row jsonb;
    row_old jsonb;
    pk json;
begin
    if tg_op = 'INSERT' or tg_op = 'UPDATE' or tg_op = 'DELETE'
    then
        row := to_jsonb(coalesce(new, old));
        row_old := to_jsonb(old);

        select json_object(array_agg(pk), array_agg(jsonb_extract_path_text(row, pk)))
        from unnest(app.table_primary_key(tg_table_schema || '.' || tg_table_name)) as t(pk)
        into pk;

        perform app.queue(
            name => tg_argv[0],
            params => json_build_object(
                'table', tg_table_schema || '.' || tg_table_name,
                'event', lower(tg_op),
                'options', tg_argv[1]::json,
                'pk', pk,
                'row', row,
                'diff', app.jsonb_diff(row_old, row),
                'diff_old', app.jsonb_diff(row, row_old),
                'role', session_user
            )
        );

        if (tg_op = 'DELETE') then
            return old;
        else
            return new;
        end if;
    else
        perform app.queue(
            tg_argv[0],
            json_build_object(
                'table', tg_table_schema || '.' || tg_table_name,
                'event', lower(tg_op),
                'options', tg_argv[1]::json,
                'pk', pk,
                'row', row,
                'diff', null,
                'role', session_user
            )
        );

        return null;
    end if;
end $$;

comment on function app.queue_trigger is E'Отправляет уведомление слушателю приложения при изменении записей\n'
                                           '(параметры: name text, options json)';

--------------------

create or replace trigger app_queue_updated_at
    before update
    on app.queue
    for each row
execute procedure app.updated_at_trigger();

comment on trigger app_queue_updated_at on app.queue is 'Установка текущей даты в updated_at';

--------------------

create or replace function app.queue_send_trigger()
    returns trigger
    language plpgsql
    security definer
as $$
    # variable_conflict use_column
begin
    perform pg_notify('app', NEW.id::text);
    return NEW;
end $$;

comment on function app.queue_send_trigger is 'Отправка уведомления фоновому процессу';

--------------------

create or replace trigger app_queue_send_notify_insert
    after insert
    on app.queue
    for each row
    when (
        NEW.state in ('new','error')
    and NEW.attempt < NEW.max_attempts
    and NEW.created_at > now() - app.config('app.queue.max_created_at_interval')::interval
    )
execute procedure app.queue_send_trigger();

comment on trigger app_queue_send_notify_insert on app.queue is 'Отправка уведомления фоновому процессу';

--------------------

create or replace trigger app_queue_send_notify_update
    after update
    on app.queue
    for each row
    when (
        NEW.state in ('new','error')
    and NEW.attempt < NEW.max_attempts
    and NEW.created_at > now() - app.config('app.queue.max_created_at_interval')::interval
    and NEW.updated_at >= OLD.updated_at + make_interval(secs => NEW.delay)
    )
execute procedure app.queue_send_trigger();

comment on trigger app_queue_send_notify_update on app.queue is 'Отправка уведомления фоновому процессу';

--------------------
