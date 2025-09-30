create type app.queue_task_state as enum (
    'new',
    'processing',
    'finished',
    'error'
);

comment on type app.queue_task_state is 'Состояния задач в очереди';
