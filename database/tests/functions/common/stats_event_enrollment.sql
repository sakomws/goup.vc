-- GOUP enrollment analytics without upstream admission offers.
begin;
select plan(2);

\set allianceID 'ea050000-0000-0000-0000-000000000001'
\set eventCategoryID 'ea050000-0000-0000-0000-000000000002'
\set eventID 'ea050000-0000-0000-0000-000000000003'
\set groupCategoryID 'ea050000-0000-0000-0000-000000000004'
\set groupID 'ea050000-0000-0000-0000-000000000005'
\set ticketTypeID 'ea050000-0000-0000-0000-000000000006'
\set directUserID 'ea050000-0000-0000-0000-000000000007'
\set invitedUserID 'ea050000-0000-0000-0000-000000000008'
\set canceledUserID 'ea050000-0000-0000-0000-000000000009'
\set waitlistUserID 'ea050000-0000-0000-0000-000000000010'

select fx_community(:'allianceID');
select fx_group_category(:'groupCategoryID', :'allianceID');
select fx_event_category(:'eventCategoryID', :'allianceID');
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');
select fx_event(:'eventID', :'groupID', :'eventCategoryID');
select fx_event_ticket_type(:'ticketTypeID', :'eventID');
select fx_user(:'directUserID');
select fx_user(:'invitedUserID');
select fx_user(:'canceledUserID');
select fx_user(:'waitlistUserID');

insert into event_attendee (
    event_id, user_id, status, checked_in, manually_invited, attendance_canceled_at
)
values
    (:'eventID', :'directUserID', 'confirmed', true, false, null),
    (:'eventID', :'invitedUserID', 'confirmed', false, true, null),
    (:'eventID', :'canceledUserID', 'attendance-canceled', false, false, current_timestamp);

insert into event_purchase (
    amount_minor, currency_code, event_id, event_ticket_type_id,
    status, ticket_title, user_id
) values (
    0, 'USD', :'eventID', :'ticketTypeID', 'completed', 'General', :'directUserID'
);

insert into event_waitlist (event_id, user_id)
values (:'eventID', :'waitlistUserID');

select is(
    (select stats_event_enrollment(e)::jsonb from event e where event_id = :'eventID'),
    '{
        "admission_offers": [],
        "cancellations": 1,
        "checked_in": 1,
        "invitation_requests": {"accepted": 0, "pending": 0, "rejected": 0},
        "registrations": 2,
        "registrations_by_path": {
            "approval": 0,
            "direct": 1,
            "organizer_invitation": 1,
            "unknown": 0,
            "waitlist": 0
        },
        "waitlist": 1
    }'::jsonb,
    'Should report GOUP enrollment paths and totals'
);

select is(
    (select stats_event_enrollment(e)::jsonb->'admission_offers' from event e where event_id = :'eventID'),
    '[]'::jsonb,
    'Should expose no unsupported admission offers'
);

select * from finish();
rollback;
