create extension if not exists hstore schema public;

create extension if not exists citext schema public;

create extension if not exists pg_trgm schema public;

create extension if not exists pgcrypto schema public;

create extension if not exists plpgsql_check schema public;

--create extension if not exists pgrowlocks schema public;

--create extension if not exists pg_stat_statements schema public;

do $$
begin
    if current_database() !~ '-test$' then
        create extension if not exists pg_cron;
    end if;
end $$;
