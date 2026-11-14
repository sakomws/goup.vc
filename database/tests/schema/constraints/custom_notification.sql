-- Tests custom notification constraints.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(8);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set cascadeAllianceID '8a110000-0000-0000-0000-000000000001'
\set cascadeNotificationID '8a110000-0000-0000-0000-000000000002'
\set allianceID '8a110000-0000-0000-0000-000000000003'
\set eventCategoryID '8a110000-0000-0000-0000-000000000004'
\set eventID '8a110000-0000-0000-0000-000000000005'
\set groupCategoryID '8a110000-0000-0000-0000-000000000006'
\set groupID '8a110000-0000-0000-0000-000000000007'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance deleted by the cascade scenario
select fx_alliance(:'cascadeAllianceID');

-- Alliance scoping the constraint scenarios
select fx_alliance(:'allianceID');

-- Alliance notification deleted through alliance cascade
insert into custom_notification (custom_notification_id, body, alliance_id, subject)
values (:'cascadeNotificationID', 'Cascade body', :'cascadeAllianceID', 'Cascade subject');

-- Event category of the scoped event
select fx_event_category(:'eventCategoryID', :'allianceID');

-- Group category of the scoped group
select fx_group_category(:'groupCategoryID', :'allianceID');

-- Group scoping the constraint scenarios
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');

-- Event scoping the constraint scenarios
select fx_event(:'eventID', :'groupID', :'eventCategoryID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should accept alliance-scoped notifications
select lives_ok(
    format(
        $$
            insert into custom_notification (body, alliance_id, subject)
            values ('Body', %L::uuid, 'Alliance scope')
        $$,
        :'allianceID'
    ),
    'Should accept alliance-scoped notifications'
);

-- Should accept event-scoped notifications
select lives_ok(
    format(
        $$
            insert into custom_notification (body, event_id, subject)
            values ('Body', %L::uuid, 'Event scope')
        $$,
        :'eventID'
    ),
    'Should accept event-scoped notifications'
);

-- Should accept group-scoped notifications
select lives_ok(
    format(
        $$
            insert into custom_notification (body, group_id, subject)
            values ('Body', %L::uuid, 'Group scope')
        $$,
        :'groupID'
    ),
    'Should accept group-scoped notifications'
);

-- Should delete alliance notifications when alliances are deleted
select lives_ok(
    format($$delete from alliance where alliance_id = %L::uuid$$, :'cascadeAllianceID'),
    'Should delete alliances with custom notifications'
);
select is(
    (
        select count(*)::int
        from custom_notification
        where custom_notification_id = :'cascadeNotificationID'
    ),
    0,
    'Should delete alliance notifications when alliances are deleted'
);

-- Should reject notifications with every scope
select throws_ok(
    format(
        $$
            insert into custom_notification (body, alliance_id, event_id, group_id, subject)
            values ('Body', %L::uuid, %L::uuid, %L::uuid, 'Every scope')
        $$,
        :'allianceID',
        :'eventID',
        :'groupID'
    ),
    '23514',
    null,
    'Should reject notifications with every scope'
);

-- Should reject notifications with two scopes
select throws_ok(
    format(
        $$
            insert into custom_notification (body, alliance_id, group_id, subject)
            values ('Body', %L::uuid, %L::uuid, 'Two scopes')
        $$,
        :'allianceID',
        :'groupID'
    ),
    '23514',
    null,
    'Should reject notifications with two scopes'
);

-- Should reject notifications without a scope
select throws_ok(
    $$
        insert into custom_notification (body, subject)
        values ('Body', 'No scope')
    $$,
    '23514',
    null,
    'Should reject notifications without a scope'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
