create or replace function app.queue_config(
    inout name text,
    out max_attempts smallint,
    out max_process_time interval,
    out delay smallint,
    out priority smallint
)
    returns record
    language plpgsql
    stable
as $$
    # variable_conflict use_variable
begin
    select t.max_attempts, t.max_process_time, t.delay, t.priority
    from (
        values
        ('send_email',  100, '30 seconds'::interval, 30, 10),
        ('reset_cache', 10,  '2 seconds'::interval,  1,  10),
        ('delete_file', 3,   '5 seconds'::interval,  1,  0)
    ) as t(name, max_attempts, max_process_time, delay, priority)
    where t.name = name
    into max_attempts, max_process_time, delay, priority;

    if not found then
        name := name;
        max_process_time := '10 seconds'::interval;
        max_attempts := 3;
        delay := 1;
        priority := 0;
    end if;
end $$;

comment on function app.queue_config is 'Настройки уведомлений по названию уведомления';

--------------

create or replace function app.queue(
    name text,
    params json = null
)
    returns app.queue
    language sql
as $$
    insert into app.queue(
           name, params, max_attempts, max_process_time, delay, priority
    )
    select $1, $2, t.max_attempts, t.max_process_time, t.delay, t.priority
    from app.queue_config($1) as t
    returning *;
$$;

comment on function app.queue is 'Отправка уведомления слушателю приложения для обработки';

--------------------

create or replace function app.queue_resend()
    returns setof uuid
    language sql
as $$
    update app.queue as n
    set state = coalesce(nullif(state, 'processing'), 'error'),
        updated_at = now()
    where n.id = any(
        select id
        from app.queue
        where state in ('new','error')
          and attempt < max_attempts
          and created_at > now() - app.config('app.queue.max_created_at_interval')::interval
          and now() >= updated_at + make_interval(secs => delay)
          or (
            state = 'processing'
            and attempt <= max_attempts
            and now() > processed_at + max_process_time
          )
        order by priority desc, updated_at
        limit 100
    )
    returning n.id;
$$;

comment on function app.queue_resend() is 'Повторно отправить все ждущие уведомления для обработки';

--------------------

create or replace function app.queue_start_process(id uuid)
    returns app.queue
    language sql
as $$
    update app.queue
    set state = 'processing',
        attempt = attempt + 1,
        processed_at = now()
    where id = $1
        and state in ('new','error')
        and attempt < max_attempts
    returning *;
$$;

comment on function app.queue_start_process is 'Начать обработку задачи';

--------------------

create or replace function app.queue_finish_process(
    id uuid,
    error text default null
)
    returns app.queue
    language sql
as $$
    with deleted as (
        delete from app.queue
        where id = $1
            and state = 'processing'
            and $2 is null
        returning *
    ),
    updated as (
        update app.queue
        set state = 'error',
            error = $2
        where id = $1
            and state = 'processing'
            and $2 is not null
        returning *
    )
    select *
    from deleted
    union all
    select *
    from updated
    limit 1;
$$;



--------------------
