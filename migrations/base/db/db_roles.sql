create role authenticator with NOINHERIT LOGIN;
create role anon with NOINHERIT NOLOGIN;
create role api with INHERIT NOLOGIN;
create role manager with INHERIT NOLOGIN;
create role admin with INHERIT NOLOGIN;
create role super with INHERIT NOLOGIN;

grant anon to api;
grant api to manager;
grant manager to admin;
grant admin to super;

grant anon to authenticator;
grant api to authenticator;
grant manager to authenticator;
grant admin to authenticator;
grant super to authenticator;

alter role authenticator set statement_timeout to '5s';
