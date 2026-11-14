-- Tests enqueueing and tracking alliance custom notifications to group teams.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(9);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID '8a100000-0000-0000-0000-000000000001'
\set foreignCategoryID '8a100000-0000-0000-0000-000000000004'
\set foreignAllianceID '8a100000-0000-0000-0000-000000000002'
\set groupAID '8a100000-0000-0000-0000-000000000005'
\set groupBID '8a100000-0000-0000-0000-000000000006'
\set multiSeatUserID '8a100000-0000-0000-0000-000000000012'
\set mutedUserID '8a100000-0000-0000-0000-000000000014'
\set optedOutUserID '8a100000-0000-0000-0000-000000000013'
\set platformCategoryID '8a100000-0000-0000-0000-000000000003'
\set senderID '8a100000-0000-0000-0000-000000000011'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance whose group teams are contacted
select fx_alliance(:'allianceID');

-- Alliance owning the foreign group category
select fx_alliance(:'foreignAllianceID');

-- Team member seated on both groups
select fx_user(:'multiSeatUserID');

-- Team member who muted group B
select fx_user(:'mutedUserID');

-- Team member who opted out of group announcements
select fx_user(:'optedOutUserID');

-- Sender who is also a team member
select fx_user(:'senderID', jsonb_build_object('username', 'sender-alliance-custom-notification'));

-- Group category of the foreign alliance
select fx_group_category(:'foreignCategoryID', :'foreignAllianceID');

-- Group category of the contacted groups
select fx_group_category(:'platformCategoryID', :'allianceID', jsonb_build_object('name', 'Platform'));

-- Group A of the contacted alliance
select fx_group(:'groupAID', :'allianceID', :'platformCategoryID');

-- Group B of the contacted alliance
select fx_group(:'groupBID', :'allianceID', :'platformCategoryID');

-- Team seats, including one person on two groups
insert into group_team (group_id, user_id, accepted, role) values
    (:'groupAID', :'multiSeatUserID', true, 'admin'),
    (:'groupAID', :'senderID', true, 'admin'),
    (:'groupBID', :'multiSeatUserID', true, 'viewer'),
    (:'groupBID', :'mutedUserID', true, 'viewer'),
    (:'groupBID', :'optedOutUserID', true, 'admin');

-- Group mute ignored by alliance custom notifications
insert into user_group_notification_mute (group_id, user_id)
values (:'groupBID', :'mutedUserID');

-- Category opt-out ignored by alliance custom notifications
insert into user_notification_opt_out (notification_category_id, user_id)
values ('group-announcements', :'optedOutUserID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return the number of distinct recipients
select is(
    enqueue_tracked_alliance_custom_notification(
        :'allianceID'::uuid,
        :'senderID'::uuid,
        jsonb_build_object(
            'group_category_ids', jsonb_build_array(:'platformCategoryID'),
            'regions', '[]'::jsonb,
            'roles', '[]'::jsonb
        ),
        '{"subject": "Alliance contact enqueue tracked"}'::jsonb,
        'Alliance contact enqueue tracked',
        'Body for the alliance group teams'
    ),
    4,
    'Should return the number of distinct recipients'
);

-- Should queue one notification per distinct person, including the sender, opted-out and muted users
select results_eq(
    $$
        select user_id, count(*)::int
        from notification
        where kind = 'alliance-custom'
        group by user_id
        order by user_id
    $$,
    format(
        $$ values
            (%L::uuid, 1),
            (%L::uuid, 1),
            (%L::uuid, 1),
            (%L::uuid, 1)
        $$,
        :'senderID',
        :'multiSeatUserID',
        :'optedOutUserID',
        :'mutedUserID'
    ),
    'Should queue one notification per distinct person, including the sender, opted-out and muted users'
);

-- Should share the template data across the queued notifications
select results_eq(
    $$
        select distinct ntd.data
        from notification n
        join notification_template_data ntd using (notification_template_data_id)
        where n.kind = 'alliance-custom'
    $$,
    $$ values ('{"subject": "Alliance contact enqueue tracked"}'::jsonb) $$,
    'Should share the template data across the queued notifications'
);

-- Should store the alliance custom notification row
select results_eq(
    $$
        select
            body,
            alliance_id,
            created_by,
            event_id,
            group_id,
            subject
        from custom_notification
        where subject = 'Alliance contact enqueue tracked'
    $$,
    format(
        $$
        values (
            'Body for the alliance group teams',
            %L::uuid,
            %L::uuid,
            null::uuid,
            null::uuid,
            'Alliance contact enqueue tracked'
        )
        $$,
        :'allianceID',
        :'senderID'
    ),
    'Should store the alliance custom notification row'
);

-- Should audit the send with the recipient count and filter names
select results_eq(
    $$
        select
            action,
            actor_user_id,
            actor_username,
            alliance_id,
            group_id,
            event_id,
            resource_type,
            resource_id,
            details
        from audit_log
        where action = 'alliance_custom_notification_sent'
    $$,
    format(
        $$
        values (
            'alliance_custom_notification_sent',
            %L::uuid,
            'sender-alliance-custom-notification',
            %L::uuid,
            null::uuid,
            null::uuid,
            'alliance',
            %L::uuid,
            jsonb_build_object(
                'group_categories', jsonb_build_array('Platform'),
                'groups', 'All',
                'recipient_count', 4,
                'regions', 'All',
                'roles', 'All',
                'subject', 'Alliance contact enqueue tracked'
            )
        )
        $$,
        :'senderID',
        :'allianceID',
        :'allianceID'
    ),
    'Should audit the send with the recipient count and filter names'
);

-- Should reject filters outside the alliance
select throws_ok(
    format(
        $$select enqueue_tracked_alliance_custom_notification(
            %L::uuid,
            %L::uuid,
            jsonb_build_object('group_category_ids', jsonb_build_array(%L)),
            '{"subject": "Foreign alliance contact"}'::jsonb,
            'Foreign alliance contact',
            'Body for foreign filters'
        )$$,
        :'allianceID',
        :'senderID',
        :'foreignCategoryID'
    ),
    'OCG01',
    'group category not found',
    'Should reject filters outside the alliance'
);

-- Should reject filters that match nobody
select throws_ok(
    format(
        $$select enqueue_tracked_alliance_custom_notification(
            %L::uuid,
            %L::uuid,
            '{"roles": ["check-in-manager"]}'::jsonb,
            '{"subject": "Empty alliance contact"}'::jsonb,
            'Empty alliance contact',
            'Body for nobody'
        )$$,
        :'allianceID',
        :'senderID'
    ),
    'OCG01',
    'no group team members match the selected filters',
    'Should reject filters that match nobody'
);

-- Should not queue notifications for rejected sends
select is(
    (select count(*)::int from notification where kind = 'alliance-custom'),
    4,
    'Should not queue notifications for rejected sends'
);

-- Should not track rejected sends
select ok(
    (select count(*) from custom_notification where alliance_id = :'allianceID') = 1
    and (select count(*) from audit_log where alliance_id = :'allianceID') = 1,
    'Should not track rejected sends'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
