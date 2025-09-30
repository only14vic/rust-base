-- https://github.com/citusdata/pg_cron
--
-- ┌───────────── min (0 - 59)
-- │ ┌────────────── hour (0 - 23)
-- │ │ ┌─────────────── day of month (1 - 31) or last day of the month ($)
-- │ │ │ ┌──────────────── month (1 - 12)
-- │ │ │ │ ┌───────────────── day of week (0 - 6) (0 to 6 are Sunday to
-- │ │ │ │ │                  Saturday, or use names; 7 is also Sunday)
-- │ │ │ │ │
-- │ │ │ │ │
-- * * * * *

do $$
begin
    if current_database() !~ '-test$' then
        perform cron.schedule('queue_resend', '10 seconds', 'select app.queue_resend()');
        perform cron.schedule('nightly_vacuum', '0 3 * * *', 'vacuum full');
        perform cron.schedule('clear_cron_run_details', '*/5 * * * *', $sql$
            delete from cron.job_run_details
            where
                (status = 'succeeded' and end_time < now() - interval '5 mins')
                or
                (status = 'failed' and end_time < now() - interval '7 days')
        $sql$);
    end if;
end $$;
