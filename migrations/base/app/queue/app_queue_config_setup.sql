insert into app.config values
    ('app.queue.max_created_at_interval', '7 days', '7 days', 'Максимальный интервал с момента создания до момента отправки уведомления на обработку'),
    ('app.queue.max_processed_at_interval', '5 minutes', '5 minutes', 'Максимальный интервал с момента начала обработки до момента определения прерванной обработки уведомления')

on conflict (name) do update set
    default_value = excluded.default_value,
    info = excluded.info;
