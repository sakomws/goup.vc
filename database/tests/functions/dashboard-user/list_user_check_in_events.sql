-- GOUP check-in listing coverage without upstream admission offers.
begin;
select plan(3);

\set allianceID '4a000000-0000-0000-0000-000000000001'
\set eventID '4a000000-0000-0000-0000-000000000002'
\set eventCategoryID '4a000000-0000-0000-0000-000000000003'
\set groupCategoryID '4a000000-0000-0000-0000-000000000004'
\set groupID '4a000000-0000-0000-0000-000000000005'
\set ticketTypeID '4a000000-0000-0000-0000-000000000006'
\set userID '4a000000-0000-0000-0000-000000000007'

select fx_community(:'allianceID');
select fx_group_category(:'groupCategoryID', :'allianceID');
select fx_event_category(:'eventCategoryID', :'allianceID');
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');
select fx_user(:'userID');
select fx_event(:'eventID', :'groupID', :'eventCategoryID', jsonb_build_object(
    'published', true,
    'starts_at', current_timestamp + interval '1 hour',
    'ends_at', current_timestamp + interval '2 hours'
));
select fx_event_ticket_type(:'ticketTypeID', :'eventID', jsonb_build_object('title', 'General'));

insert into event_attendee (event_id, user_id, status, checked_in)
values (:'eventID', :'userID', 'confirmed', true);

insert into event_purchase (
    amount_minor, currency_code, event_id, event_ticket_type_id,
    status, ticket_title, user_id
) values (
    0, 'USD', :'eventID', :'ticketTypeID', 'completed', 'General', :'userID'
);

select is(
    (list_user_check_in_events(:'userID')::jsonb->0->>'event_id'),
    :'eventID',
    'Should list the user check-in event'
);

select is(
    (list_user_check_in_events(:'userID')::jsonb->0->>'ticket_title'),
    'General',
    'Should use the GOUP purchase ticket snapshot'
);

select ok(
    not list_user_check_in_events(:'userID')::text like '%check_in_code%',
    'Should not expose the raw check-in credential'
);

select * from finish();
rollback;
