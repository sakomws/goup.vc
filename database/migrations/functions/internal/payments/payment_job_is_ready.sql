-- Returns whether the domain row of a payment job is in a state a worker can act on.
create or replace function payment_job_is_ready(p_job payment_job)
returns boolean as $$
    select case p_job.kind
        -- Refunds pinned to a terminal provider failure wait for operator recovery
        when 'event-purchase-refund' then exists (
            select 1
            from event_purchase_refund epr
            where epr.payment_job_id = p_job.payment_job_id
            and not (epr.status = 'provider-failed' and epr.terminal_failure)
        )
        else false
    end;
$$ language sql stable;
