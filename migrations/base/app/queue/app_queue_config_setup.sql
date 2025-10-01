insert into app.config values
    ('app.queue.max_created_at_interval', '2 days', '2 days', 'Максимальный интервал с момента создания до момента отправки уведомления на обработку')

on conflict (name) do update set
    default_value = excluded.default_value,
    info = excluded.info;
