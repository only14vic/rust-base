create or replace function auth.current_user_is_usesuper()
    returns bool
    language sql
    stable
    parallel safe
as $$
    select exists(
        select from pg_user
        where usename = current_user
          and usesuper = true
    );
$$;

comment on function auth.current_user_is_usesuper is 'Имеет ли текущий пользователь роль super в базе';

-----------
