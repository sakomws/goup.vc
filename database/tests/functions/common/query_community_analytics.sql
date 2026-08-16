begin;
select plan(5);

\set allianceID '6c010000-0000-0000-0000-000000000001'
\set eventCategoryID '6c010000-0000-0000-0000-000000000002'
\set groupCategoryID '6c010000-0000-0000-0000-000000000003'
\set japanGroupID '6c010000-0000-0000-0000-000000000004'
\set germanyGroupID '6c010000-0000-0000-0000-000000000005'
\set deletedGroupID '6c010000-0000-0000-0000-000000000006'
\set japanEventID '6c010000-0000-0000-0000-000000000007'
\set germanyEventID '6c010000-0000-0000-0000-000000000008'
\set testEventID '6c010000-0000-0000-0000-000000000009'
\set canceledEventID '6c010000-0000-0000-0000-00000000000a'
\set unpublishedEventID '6c010000-0000-0000-0000-00000000000b'
\set deletedEventID '6c010000-0000-0000-0000-00000000000c'
\set user1ID '6c010000-0000-0000-0000-00000000000d'
\set user2ID '6c010000-0000-0000-0000-00000000000e'
\set user3ID '6c010000-0000-0000-0000-00000000000f'
\set user4ID '6c010000-0000-0000-0000-000000000010'

insert into alliance (
    alliance_id, name, display_name, description, banner_mobile_url, banner_url, logo_url
) values (
    :'allianceID', 'analytics-alliance', 'Analytics Alliance', 'Analytics test alliance',
    'https://example.com/banner-mobile.png', 'https://example.com/banner.png', 'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Meetup');

insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Meetup');

insert into "group" (
    group_id, alliance_id, group_category_id, name, slug, country_code, active, deleted
) values
    (:'japanGroupID', :'allianceID', :'groupCategoryID', 'Japan Community', 'japan-community', 'JP', true, false),
    (:'germanyGroupID', :'allianceID', :'groupCategoryID', 'Germany Community', 'germany-community', 'DE', true, false),
    (:'deletedGroupID', :'allianceID', :'groupCategoryID', 'Deleted Community', 'deleted-community', 'JP', false, true);

insert into "user" (user_id, auth_hash, email, username) values
    (:'user1ID', gen_random_bytes(32), 'analytics-user1@example.com', 'analytics-user1'),
    (:'user2ID', gen_random_bytes(32), 'analytics-user2@example.com', 'analytics-user2'),
    (:'user3ID', gen_random_bytes(32), 'analytics-user3@example.com', 'analytics-user3'),
    (:'user4ID', gen_random_bytes(32), 'analytics-user4@example.com', 'analytics-user4');

insert into event (
    event_id, group_id, event_category_id, event_kind_id, name, slug, description, timezone,
    starts_at, venue_country_code, published, canceled, deleted, test_event
) values
    (:'japanEventID', :'japanGroupID', :'eventCategoryID', 'in-person', 'Japan Event', 'japan-event',
        'Published Japan event', 'UTC', '2025-02-01 10:00:00+00', 'JP', true, false, false, false),
    (:'germanyEventID', :'germanyGroupID', :'eventCategoryID', 'in-person', 'Germany Event', 'germany-event',
        'Published Germany event', 'UTC', '2025-03-01 10:00:00+00', 'DE', true, false, false, false),
    (:'testEventID', :'japanGroupID', :'eventCategoryID', 'in-person', 'Test Event', 'test-event',
        'Excluded test event', 'UTC', '2025-04-01 10:00:00+00', 'JP', true, false, false, true),
    (:'canceledEventID', :'japanGroupID', :'eventCategoryID', 'in-person', 'Canceled Event', 'canceled-event',
        'Excluded canceled event', 'UTC', '2025-05-01 10:00:00+00', 'JP', true, true, false, false),
    (:'unpublishedEventID', :'japanGroupID', :'eventCategoryID', 'in-person', 'Draft Event', 'draft-event',
        'Excluded draft event', 'UTC', '2025-06-01 10:00:00+00', 'JP', false, false, false, false),
    (:'deletedEventID', :'deletedGroupID', :'eventCategoryID', 'in-person', 'Deleted Event', 'deleted-event',
        'Excluded deleted event', 'UTC', '2025-07-01 10:00:00+00', 'JP', false, false, true, false);

insert into event_attendee (event_id, user_id, status) values
    (:'japanEventID', :'user1ID', 'confirmed'),
    (:'japanEventID', :'user2ID', 'confirmed'),
    (:'japanEventID', :'user3ID', 'invitation-pending'),
    (:'germanyEventID', :'user1ID', 'confirmed'),
    (:'testEventID', :'user1ID', 'confirmed'),
    (:'canceledEventID', :'user1ID', 'confirmed'),
    (:'unpublishedEventID', :'user1ID', 'confirmed'),
    (:'deletedEventID', :'user1ID', 'confirmed');

select is(
    query_community_analytics('2025-01-01 00:00:00+00', '2026-01-01 00:00:00+00') - 'period',
    jsonb_build_object(
        'event_count', 2,
        'attendee_count', 3,
        'by_country', jsonb_build_array(
            jsonb_build_object('country_code', 'JP', 'event_count', 1, 'attendee_count', 2),
            jsonb_build_object('country_code', 'DE', 'event_count', 1, 'attendee_count', 1)
        ),
        'top_communities', jsonb_build_array(
            jsonb_build_object('community_name', 'Japan Community', 'event_count', 1, 'attendee_count', 2),
            jsonb_build_object('community_name', 'Germany Community', 'event_count', 1, 'attendee_count', 1)
        )
    ),
    'Returns only eligible public event and confirmed-attendance aggregates'
);

select is(
    query_community_analytics(
        '2025-01-01 00:00:00+00', '2026-01-01 00:00:00+00', array['jp']
    ) - 'period',
    jsonb_build_object(
        'event_count', 1,
        'attendee_count', 2,
        'by_country', jsonb_build_array(
            jsonb_build_object('country_code', 'JP', 'event_count', 1, 'attendee_count', 2)
        ),
        'top_communities', jsonb_build_array(
            jsonb_build_object('community_name', 'Japan Community', 'event_count', 1, 'attendee_count', 2)
        )
    ),
    'Filters countries case-insensitively'
);

select is(
    query_community_analytics(
        '2025-01-01 00:00:00+00', '2026-01-01 00:00:00+00', null, 'analytics-alliance'
    )->'top_communities',
    jsonb_build_array(
        jsonb_build_object('community_name', 'Japan Community', 'event_count', 1, 'attendee_count', 2),
        jsonb_build_object('community_name', 'Germany Community', 'event_count', 1, 'attendee_count', 1)
    ),
    'Filters by alliance without exposing identifiers'
);

select ok(
    not (
        query_community_analytics('2025-01-01 00:00:00+00', '2026-01-01 00:00:00+00')
        ?| array['user_id', 'email', 'event_id', 'group_id']
    ),
    'Does not return PII or internal identifiers'
);

select throws_ok(
    $$select query_community_analytics('2026-01-01 00:00:00+00', '2025-01-01 00:00:00+00')$$,
    'start_at must be before end_at',
    'Rejects invalid time ranges'
);

select * from finish();
rollback;
