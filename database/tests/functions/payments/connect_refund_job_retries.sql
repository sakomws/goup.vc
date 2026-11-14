-- Connect-specific retry coverage for the durable refund payment-job queue.
begin;
select plan(9);

\set actorID 'd2020000-0000-0000-0000-000000000001'
\set buyerID 'd2020000-0000-0000-0000-000000000002'
\set allianceID 'd2020000-0000-0000-0000-000000000003'
\set eventCategoryID 'd2020000-0000-0000-0000-000000000004'
\set eventID 'd2020000-0000-0000-0000-000000000005'
\set groupCategoryID 'd2020000-0000-0000-0000-000000000006'
\set groupID 'd2020000-0000-0000-0000-000000000007'
\set purchaseID 'd2020000-0000-0000-0000-000000000008'
\set refundRequestID 'd2020000-0000-0000-0000-000000000009'
\set ticketTypeID 'd2020000-0000-0000-0000-000000000010'

select fx_community(:'allianceID');
select fx_group_category(:'groupCategoryID', :'allianceID');
select fx_event_category(:'eventCategoryID', :'allianceID');
select fx_user(:'actorID');
select fx_user(:'buyerID');
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');
select fx_event(:'eventID', :'groupID', :'eventCategoryID', jsonb_build_object(
    'payment_currency_code', 'USD',
    'published', true,
    'starts_at', current_timestamp + interval '1 day'
));
select fx_event_ticket_type(:'ticketTypeID', :'eventID');

insert into event_purchase (
    amount_minor,
    currency_code,
    event_id,
    event_purchase_id,
    event_ticket_type_id,
    payment_provider_id,
    provider_payment_reference,
    status,
    ticket_title,
    user_id
) values (
    2500,
    'USD',
    :'eventID',
    :'purchaseID',
    :'ticketTypeID',
    'stripe',
    'pi_connect_retry',
    'refund-requested',
    'General admission',
    :'buyerID'
);

insert into event_refund_request (
    event_purchase_id,
    event_refund_request_id,
    requested_by_user_id,
    status
) values (
    :'purchaseID',
    :'refundRequestID',
    :'buyerID',
    'pending'
);

select lives_ok(
    format(
        $$select queue_event_refund_request_approval(%L, %L, %L, 'approved')$$,
        :'actorID', :'groupID', :'purchaseID'
    ),
    'Should queue Connect refund work'
);

select isnt(
    claim_payment_job('event-purchase-refund', 'stripe'),
    null,
    'Should claim Connect refund work'
);

select lives_ok(
    format(
        $$select record_payment_job_failure(pj.payment_job_id, pj.claim_id, 'temporary provider error')
          from payment_job pj
          join event_purchase_refund epr using (payment_job_id)
          where epr.event_purchase_id = %L$$,
        :'purchaseID'
    ),
    'Should release a failed Connect refund claim'
);

select results_eq(
    format(
        $$select pj.attempt_count, pj.claim_id is null, pj.failure_message, pj.status
          from payment_job pj
          join event_purchase_refund epr using (payment_job_id)
          where epr.event_purchase_id = %L$$,
        :'purchaseID'
    ),
    $$values (1, true, 'temporary provider error'::text, 'failed'::text)$$,
    'Should retain retryable failure state'
);

update payment_job pj
set next_attempt_at = current_timestamp
from event_purchase_refund epr
where pj.payment_job_id = epr.payment_job_id
and epr.event_purchase_id = :'purchaseID';

select isnt(
    claim_payment_job('event-purchase-refund', 'stripe'),
    null,
    'Should reclaim retryable Connect work'
);

update payment_job pj
set claimed_at = current_timestamp - interval '16 minutes'
from event_purchase_refund epr
where pj.payment_job_id = epr.payment_job_id
and epr.event_purchase_id = :'purchaseID';

select is(
    requeue_stale_payment_job_claims(),
    1,
    'Should release a stale Connect worker claim'
);

select results_eq(
    format(
        $$select pj.attempt_count, pj.claim_id is null, pj.status
          from payment_job pj
          join event_purchase_refund epr using (payment_job_id)
          where epr.event_purchase_id = %L$$,
        :'purchaseID'
    ),
    $$values (2, true, 'failed'::text)$$,
    'Should preserve the stale claim attempt count'
);

update payment_job pj
set attempt_count = payment_job_max_attempts(),
    status = 'failed'
from event_purchase_refund epr
where pj.payment_job_id = epr.payment_job_id
and epr.event_purchase_id = :'purchaseID';

select lives_ok(
    format(
        $$select requeue_event_purchase_refund(%L, %L)$$,
        :'groupID', :'purchaseID'
    ),
    'Should allow an administrator to requeue an exhausted Connect refund'
);

select results_eq(
    format(
        $$select pj.attempt_count, pj.failure_message, pj.status
          from payment_job pj
          join event_purchase_refund epr using (payment_job_id)
          where epr.event_purchase_id = %L$$,
        :'purchaseID'
    ),
    $$values (0, null::text, 'pending'::text)$$,
    'Should reset exhausted Connect refund work'
);

select * from finish();
rollback;
