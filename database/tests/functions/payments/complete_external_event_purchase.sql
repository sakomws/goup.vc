-- Connect-compatible coverage for organizer-confirmed external payments.
begin;
select plan(5);

\set actorID 'e07a0000-0000-0000-0000-000000000001'
\set attendeeID 'e07a0000-0000-0000-0000-000000000002'
\set allianceID 'e07a0000-0000-0000-0000-000000000003'
\set purchaseID 'e07a0000-0000-0000-0000-000000000004'
\set eventCategoryID 'e07a0000-0000-0000-0000-000000000005'
\set eventID 'e07a0000-0000-0000-0000-000000000006'
\set ticketTypeID 'e07a0000-0000-0000-0000-000000000007'
\set groupCategoryID 'e07a0000-0000-0000-0000-000000000008'
\set groupID 'e07a0000-0000-0000-0000-000000000009'

select fx_community(:'allianceID');
select fx_group_category(:'groupCategoryID', :'allianceID');
select fx_event_category(:'eventCategoryID', :'allianceID');
select fx_user(:'actorID', jsonb_build_object('username', 'external-actor'));
select fx_user(:'attendeeID');
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');
select fx_event(:'eventID', :'groupID', :'eventCategoryID', jsonb_build_object(
    'published', true,
    'starts_at', current_timestamp + interval '7 days'
));
select fx_event_ticket_type(:'ticketTypeID', :'eventID');

insert into event_purchase (
    amount_minor,
    charge_model,
    currency_code,
    event_id,
    event_purchase_id,
    event_ticket_type_id,
    hold_expires_at,
    status,
    ticket_title,
    user_id
) values (
    5000,
    'external',
    'USD',
    :'eventID',
    :'purchaseID',
    :'ticketTypeID',
    current_timestamp + interval '2 days',
    'pending',
    'General admission',
    :'attendeeID'
);

select is(
    complete_external_event_purchase(
        :'actorID', :'groupID', :'eventID', :'attendeeID', 'bank transfer ref 123'
    ),
    jsonb_build_object(
        'alliance_id', :'allianceID'::uuid,
        'event_id', :'eventID'::uuid,
        'event_purchase_id', :'purchaseID'::uuid,
        'user_id', :'attendeeID'::uuid
    ),
    'Should complete a pending external purchase'
);

select results_eq(
    format(
        $$select status, external_payment_details, hold_expires_at is null
          from event_purchase where event_purchase_id = %L$$,
        :'purchaseID'
    ),
    $$values ('completed'::text, 'bank transfer ref 123'::text, true)$$,
    'Should persist external payment completion'
);

select results_eq(
    format(
        $$select status from event_attendee where event_id = %L and user_id = %L$$,
        :'eventID', :'attendeeID'
    ),
    $$values ('confirmed'::text)$$,
    'Should confirm the attendee'
);

select is(
    (
        select count(*)::int
        from audit_log
        where action = 'event_external_payment_confirmed'
        and resource_id = :'eventID'
    ),
    1,
    'Should audit external payment completion'
);

select throws_ok(
    format(
        $$select complete_external_event_purchase(%L, %L, %L, %L, 'again')$$,
        :'actorID', :'groupID', :'eventID', :'attendeeID'
    ),
    'pending external purchase not found',
    'Should reject completion without a pending external purchase'
);

select * from finish();
rollback;
