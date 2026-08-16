begin;
select plan(13);

\set primaryAlliance '71000000-0000-0000-0000-000000000001'
\set cohostAlliance '71000000-0000-0000-0000-000000000002'
\set primaryCategory '72000000-0000-0000-0000-000000000001'
\set cohostCategory '72000000-0000-0000-0000-000000000002'
\set eventCategory '72000000-0000-0000-0000-000000000003'
\set primaryGroup '73000000-0000-0000-0000-000000000001'
\set cohostGroup '73000000-0000-0000-0000-000000000002'
\set primaryUser '74000000-0000-0000-0000-000000000001'
\set cohostUser '74000000-0000-0000-0000-000000000002'
\set eventID '75000000-0000-0000-0000-000000000001'

insert into "user" (user_id, auth_hash, email, username) values
    (:'primaryUser', 'cohost-primary', 'cohost-primary@example.com', 'cohost-primary'),
    (:'cohostUser', 'cohost-target', 'cohost-target@example.com', 'cohost-target');

insert into alliance (alliance_id, name, display_name, description, logo_url, banner_url, banner_mobile_url) values
    (:'primaryAlliance', 'cohost-primary', 'Co-host primary', 'Primary test alliance', 'https://example.com/logo.png', 'https://example.com/banner.png', 'https://example.com/mobile.png'),
    (:'cohostAlliance', 'cohost-target', 'Co-host target', 'Target test alliance', 'https://example.com/logo.png', 'https://example.com/banner.png', 'https://example.com/mobile.png');

insert into group_category (group_category_id, alliance_id, name) values
    (:'primaryCategory', :'primaryAlliance', 'Primary co-host category'),
    (:'cohostCategory', :'cohostAlliance', 'Target co-host category');
insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategory', :'primaryAlliance', 'Co-host events');

insert into "group" (group_id, alliance_id, group_category_id, name, slug, location) values
    (:'primaryGroup', :'primaryAlliance', :'primaryCategory', 'Primary group', 'cohost-primary-group', st_setsrid(st_makepoint(-122.4194, 37.7749), 4326)::geography),
    (:'cohostGroup', :'cohostAlliance', :'cohostCategory', 'Target group', 'cohost-target-group', st_setsrid(st_makepoint(-122.4094, 37.7849), 4326)::geography);
insert into group_team (group_id, user_id, role, accepted) values
    (:'primaryGroup', :'primaryUser', 'admin', true),
    (:'cohostGroup', :'cohostUser', 'admin', true);
insert into group_member (group_id, user_id) values
    (:'cohostGroup', :'cohostUser');

insert into event (
    event_id, event_category_id, event_kind_id, group_id, name, slug, description,
    timezone, starts_at, published, location
) values (
    :'eventID', :'eventCategory', 'in-person', :'primaryGroup', 'Co-host event',
    'cohost-event', 'A co-host event', 'UTC', current_timestamp + interval '2 days', true,
    st_setsrid(st_makepoint(-122.4194, 37.7749), 4326)::geography
);

select has_table('event_cohost', 'co-host invitation history is persisted');
select has_table('group_cohost_notification_preference', 'co-host notification preference is persisted');
select has_table('event_cohost_delivery', 'calendar delivery idempotency is persisted');
select has_function('request_event_cohost', array['uuid', 'uuid', 'uuid', 'text']::name[]);
select has_function('claim_event_cohost_delivery', array['uuid', 'uuid', 'uuid', 'text']::name[]);

select ok(
    (list_event_cohost_candidates(:'primaryUser', :'eventID', 20)::jsonb @> jsonb_build_array(jsonb_build_object('group_id', :'cohostGroup'::uuid))),
    'geo discovery returns a nearby cross-alliance group'
);
select isnt(
    request_event_cohost(:'primaryUser', :'eventID', :'cohostGroup', 'Please co-host'),
    null::uuid,
    'a primary organizer can invite a cross-alliance co-host'
);
select is(
    request_event_cohost(:'primaryUser', :'eventID', :'cohostGroup', 'retry'),
    (select event_cohost_id from event_cohost where event_id = :'eventID' and cohost_group_id = :'cohostGroup'),
    'repeated invitation requests are idempotent'
);
select lives_ok(
    $$select decide_event_cohost(
        '74000000-0000-0000-0000-000000000002',
        (select event_cohost_id from event_cohost where event_id = '75000000-0000-0000-0000-000000000001'),
        true
    )$$,
    'a target organizer can approve its invitation'
);
select ok(
    event_is_visible_to_group(:'eventID', :'cohostGroup'),
    'an approved co-host event is visible in the co-host feed'
);
select ok(
    get_group_upcoming_events(:'cohostAlliance', 'cohost-target-group', array['in-person'], 10)::jsonb
        @> jsonb_build_array(jsonb_build_object('event_id', :'eventID'::uuid)),
    'the standard public group feed includes approved cross-alliance co-host events'
);
select is(
    claim_event_cohost_delivery(:'eventID', :'cohostGroup', :'cohostUser', 'calendar-invite'),
    true,
    'the first calendar invite claim succeeds'
);
select is(
    claim_event_cohost_delivery(:'eventID', :'cohostGroup', :'cohostUser', 'calendar-invite'),
    false,
    'a retried calendar invite claim is idempotently suppressed'
);

select * from finish();
rollback;
