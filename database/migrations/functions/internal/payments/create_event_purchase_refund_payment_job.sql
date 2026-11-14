-- Creates the shared lifecycle row for a newly queued provider refund.
create or replace function create_event_purchase_refund_payment_job()
returns trigger as $$
begin
    if new.payment_job_id is null then
        new.payment_job_id := gen_random_uuid();
    end if;

    insert into payment_job (
        payment_job_id,
        event_purchase_id,
        idempotency_key,
        kind,
        payment_provider_id
    ) values (
        new.payment_job_id,
        new.event_purchase_id,
        new.idempotency_key,
        'event-purchase-refund',
        new.payment_provider_id
    );

    return new;
end;
$$ language plpgsql;
