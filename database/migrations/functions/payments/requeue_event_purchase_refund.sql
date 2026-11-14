-- Requeues an exhausted retryable refund after an administrator retries it.
create or replace function requeue_event_purchase_refund(
    p_group_id uuid,
    p_event_purchase_id uuid
)
returns void as $$
begin
    -- Requeue only exhausted failures that were not terminal provider results
    update payment_job pj
    set
        attempt_count = 0,
        failure_message = null,
        next_attempt_at = current_timestamp,
        status = 'pending',
        updated_at = current_timestamp
    from event_purchase_refund epr
    join event_purchase ep on ep.event_purchase_id = epr.event_purchase_id
    join event e on e.event_id = ep.event_id
    where pj.payment_job_id = epr.payment_job_id
    and e.group_id = p_group_id
    and ep.event_purchase_id = p_event_purchase_id
    and epr.status in ('provider-failed', 'provider-pending')
    and not epr.terminal_failure
    and payment_job_is_exhausted(pj);

    if not found then
        raise exception 'retryable event purchase refund not found';
    end if;
end;
$$ language plpgsql;
