create schema if not exists app;
create schema if not exists api;
create schema if not exists auth;
create schema if not exists jwt;
create schema if not exists postgrest;

grant usage on schema postgrest to authenticator, anon;
grant usage on schema app to anon;
grant usage on schema api to anon;
grant usage on schema auth to anon;
