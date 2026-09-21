begin;
select plan(5);

\set allianceID '7b440000-0000-0000-0000-000000000001'
\set eventCategoryID '7b440000-0000-0000-0000-000000000002'
\set eventID '7b440000-0000-0000-0000-000000000003'
\set groupAdminID '7b440000-0000-0000-0000-000000000004'
\set groupCategoryID '7b440000-0000-0000-0000-000000000005'
\set sourceGroupID '7b440000-0000-0000-0000-000000000006'
\set targetGroupID '7b440000-0000-0000-0000-000000000007'
\set allianceLeadID '7b440000-0000-0000-0000-000000000008'

insert into alliance (
    alliance_id,
    name,
    display_name,
    description,
    banner_mobile_url,
    banner_url,
    logo_url
) values (
    :'allianceID',
    'event-move-alliance',
    'Event Move Alliance',
    'Tests event move permissions',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Technology');

insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Meetup');

insert into "user" (user_id, auth_hash, email, email_verified, username, name) values
    (:'groupAdminID', gen_random_bytes(32), 'group-admin@example.com', true, 'group-admin', 'Group Admin'),
    (:'allianceLeadID', gen_random_bytes(32), 'alliance-lead@example.com', true, 'alliance-lead', 'Alliance Lead');

insert into "group" (group_id, alliance_id, group_category_id, name, slug) values
    (:'sourceGroupID', :'allianceID', :'groupCategoryID', 'Source Group', 'source-group'),
    (:'targetGroupID', :'allianceID', :'groupCategoryID', 'Target Group', 'target-group');

insert into group_team (group_id, user_id, role, accepted)
values (:'sourceGroupID', :'groupAdminID', 'admin', true);

insert into alliance_team (alliance_id, user_id, role, accepted)
values (:'allianceID', :'allianceLeadID', 'admin', true);

insert into event (
    event_id,
    group_id,
    name,
    slug,
    description,
    timezone,
    event_category_id,
    event_kind_id
) values (
    :'eventID',
    :'sourceGroupID',
    'Event to Move',
    'event-to-move',
    'Event move permission test',
    'UTC',
    :'eventCategoryID',
    'in-person'
);

select is(
    list_group_move_targets(
        :'groupAdminID'::uuid,
        :'allianceID'::uuid,
        :'sourceGroupID'::uuid
    )::jsonb,
    '[]'::jsonb,
    'Group-only admins cannot list event move targets'
);

select throws_ok(
    format(
        $$select move_event('%s'::uuid, '%s'::uuid, '%s'::uuid, '%s'::uuid)$$,
        :'groupAdminID',
        :'sourceGroupID',
        :'eventID',
        :'targetGroupID'
    ),
    'alliance group management permission required',
    'Group-only admins cannot move events'
);

select is(
    list_group_move_targets(
        :'allianceLeadID'::uuid,
        :'allianceID'::uuid,
        :'sourceGroupID'::uuid
    )::jsonb,
    jsonb_build_array(jsonb_build_object(
        'group_id', :'targetGroupID'::uuid,
        'name', 'Target Group',
        'slug', 'target-group'
    )),
    'Alliance group managers can list event move targets'
);

select lives_ok(
    format(
        $$select move_event('%s'::uuid, '%s'::uuid, '%s'::uuid, '%s'::uuid)$$,
        :'allianceLeadID',
        :'sourceGroupID',
        :'eventID',
        :'targetGroupID'
    ),
    'Alliance group managers can move events'
);

select is(
    (select group_id from event where event_id = :'eventID'::uuid),
    :'targetGroupID'::uuid,
    'Event is moved to the selected group'
);

select * from finish();
rollback;
