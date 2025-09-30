create table if not exists app.queue (
    id uuid primary key default gen_random_uuid(),
    state app.queue_task_state not null default 'new'::app.queue_task_state,
    priority smallint not null default 0 check (priority >= 0),
    delay smallint not null default 0 check (delay >= 0),
    attempt smallint not null default 0 check (attempt >= 0 and attempt <= max_attempts),
    max_attempts smallint not null default 3 check (max_attempts > 0),
    name text not null,
    params json,
    error text default null,
    processed_at timestamptz default null check (state != 'new' and processed_at is not null or processed_at is null),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

comment on table app.queue is 'Список задач в очереди';

create index if not exists app_queue_filter_idx on app.queue(state, priority, updated_at);

--------------------
