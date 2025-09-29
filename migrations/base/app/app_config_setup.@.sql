insert into app.config values
    ('app.roles', app.config('app.roles', array['super','admin','manager','api']::text), array['super','admin','manager','api']::text, 'Допустимые роли')

on conflict (name) do update set
    default_value = excluded.default_value,
    info = excluded.info;

-----------------------
