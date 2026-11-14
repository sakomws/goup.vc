-- GOUP ticket analytics without upstream admission offers.
begin;
select plan(2);

\set allianceID 'ea060000-0000-0000-0000-000000000001'
\set eventCategoryID 'ea060000-0000-0000-0000-000000000002'
\set eventID 'ea060000-0000-0000-0000-000000000003'
\set groupCategoryID 'ea060000-0000-0000-0000-000000000004'
\set groupID 'ea060000-0000-0000-0000-000000000005'
\set ticketTypeID 'ea060000-0000-0000-0000-000000000006'
\set userID 'ea060000-0000-0000-0000-000000000007'

select fx_community(:'allianceID');
select fx_group_category(:'groupCategoryID', :'allianceID');
select fx_event_category(:'eventCategoryID', :'allianceID');
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');
select fx_event(:'eventID', :'groupID', :'eventCategoryID');
select fx_event_ticket_type(:'ticketTypeID', :'eventID', jsonb_build_object(
    'seats_total', 10,
    'title', 'General'
));
select fx_user(:'userID');

insert into event_purchase (
    amount_minor, currency_code, event_id, event_ticket_type_id,
    status, ticket_title, user_id
) values (
    1000, 'USD', :'eventID', :'ticketTypeID', 'completed', 'General', :'userID'
);

select is(
    (select stats_event_tickets(e)::jsonb->'ticket_types'->0 from event e where event_id = :'eventID'),
    jsonb_build_object(
        'active', true,
        'allocated', 1,
        'event_ticket_type_id', :'ticketTypeID'::uuid,
        'order', 1,
        'remaining', 9,
        'seats_total', 10,
        'sold', 1,
        'title', 'General'
    ),
    'Should report GOUP sold and allocated ticket seats'
);

select is(
    (select stats_event_tickets(e)::jsonb->'discount_codes' from event e where event_id = :'eventID'),
    '[]'::jsonb,
    'Should return an empty discount list when none exist'
);

select * from finish();
rollback;
