create or replace function app.op_contains(public.citext, text)
    returns bool
    language sql
    stable
    parallel safe
as $$
    select $1::text ilike any(format('{%s}', replace(trim(both '{}' from $2), '*', '%'))::text[]);
$$;

do $$
begin
    if to_regoperator('@>(citext,text)') is null then
        create operator @> (
            leftarg = public.citext,
            rightarg = text,
            function = app.op_contains,
            commutator = @>);
    end if;
end $$;

--------------

create or replace function app.op_contains(text, text)
    returns bool
    language sql
    stable
    parallel safe
as $$
    select $1 ilike any(format('{%s}', replace(trim(both '{}' from $2), '*', '%'))::text[]);
$$;

do $$
begin
    if to_regoperator('@>(text,text)') is null then
        create operator @> (
            leftarg = text,
            rightarg = text,
            function = app.op_contains,
            commutator = @>);
    end if;
end $$;

------------

create or replace function app.op_contains(timestamptz, text)
    returns bool
    language sql
    stable
    parallel safe
as $$
    select app.to_text($1) ilike any(format('{%s}', replace(trim(both '{}' from $2), '*', '%'))::text[]);
$$;

do $$
begin
    if to_regoperator('@>(timestamptz,text)') is null then
        create operator @> (
            leftarg = timestamptz,
            rightarg = text,
            function = app.op_contains,
            commutator = @>);
    end if;
end $$;

------------

create or replace function app.op_merge_jsonb(jsonb, jsonb)
    returns jsonb
    language sql
    stable
    parallel safe
as $$
    select app.jsonb_merge($1, $2);
$$;

do $$
begin
    if to_regoperator('+(jsonb,jsonb)') is null then
        create operator + (
            leftarg = jsonb,
            rightarg = jsonb,
            function = app.op_merge_jsonb,
            commutator = +);
    end if;
end $$;

------------
