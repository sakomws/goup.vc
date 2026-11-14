-- Move Connect refund processing onto the shared durable payment-job lifecycle.
-- Destination charges refund their application fee through the refund request itself,
-- so GOUP does not need the upstream direct-charge fee-adjustment or credit-note jobs.

drop function if exists claim_event_purchase_refund(text);
drop function if exists record_event_purchase_refund_retryable_failure(uuid, uuid, text);
drop function if exists requeue_stale_event_purchase_refund_claims();

create table payment_job (
    payment_job_id uuid primary key default gen_random_uuid(),
    attempt_count int default 0 not null check (attempt_count >= 0),
    created_at timestamptz default current_timestamp not null,
    event_purchase_id uuid not null references event_purchase,
    idempotency_key text not null unique check (btrim(idempotency_key) <> ''),
    kind text not null,
    next_attempt_at timestamptz default current_timestamp not null,
    payment_provider_id text not null references payment_provider,
    status text default 'pending' not null,
    updated_at timestamptz default current_timestamp not null,

    claim_id uuid,
    claimed_at timestamptz,
    completed_at timestamptz,
    failure_message text check (btrim(failure_message) <> ''),

    constraint payment_job_claim_chk check (
        (status = 'processing' and claim_id is not null and claimed_at is not null)
        or (status <> 'processing' and claim_id is null and claimed_at is null)
    ),
    constraint payment_job_completed_chk check (
        (status = 'completed' and completed_at is not null)
        or (status <> 'completed' and completed_at is null)
    ),
    constraint payment_job_kind_chk check (kind = 'event-purchase-refund'),
    constraint payment_job_status_chk check (
        status = any(array['completed', 'failed', 'pending', 'processing']::text[])
    )
);

create index payment_job_claimed_at_idx
    on payment_job (claimed_at)
    where claimed_at is not null;
create index payment_job_event_purchase_id_idx on payment_job (event_purchase_id);
create index payment_job_work_idx on payment_job (kind, status, next_attempt_at);

alter table event_purchase_refund
    add column payment_job_id uuid,
    add column if not exists recovery_completed_at timestamptz,
    add column if not exists recovery_completed_by_user_id uuid references "user",
    add column if not exists recovery_note text,
    add column if not exists recovery_reference text;

update event_purchase_refund
set payment_job_id = gen_random_uuid();

insert into payment_job (
    payment_job_id,
    attempt_count,
    created_at,
    event_purchase_id,
    idempotency_key,
    kind,
    next_attempt_at,
    payment_provider_id,
    status,
    updated_at,
    claim_id,
    claimed_at,
    completed_at,
    failure_message
)
select
    epr.payment_job_id,
    case
        when epr.provider_refunded_at is not null and epr.finalized_at is null then 0
        else epr.attempt_count
    end,
    epr.created_at,
    epr.event_purchase_id,
    epr.idempotency_key,
    'event-purchase-refund',
    case
        when epr.provider_refunded_at is not null and epr.finalized_at is null
            then current_timestamp
        else epr.next_attempt_at
    end,
    epr.payment_provider_id,
    case
        when epr.status = 'finalized' then 'completed'
        when epr.status = 'processing' then 'processing'
        when epr.status = 'provider-failed' then 'failed'
        else 'pending'
    end,
    epr.updated_at,
    epr.claim_id,
    epr.claimed_at,
    case when epr.status = 'finalized' then epr.finalized_at end,
    case
        when epr.status = 'finalized'
          or (epr.provider_refunded_at is not null and epr.finalized_at is null) then null
        else epr.failure_message
    end
from event_purchase_refund epr;

alter table event_purchase_refund
    alter column payment_job_id set not null,
    add constraint event_purchase_refund_payment_job_id_fkey
        foreign key (payment_job_id) references payment_job,
    add constraint event_purchase_refund_payment_job_id_key unique (payment_job_id);

-- Every future refund receives its lifecycle row in the same transaction.
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

create trigger event_purchase_refund_payment_job_create
before insert on event_purchase_refund
for each row
execute function create_event_purchase_refund_payment_job();
