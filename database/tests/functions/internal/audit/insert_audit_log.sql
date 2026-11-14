-- Tests inserting audit log entries.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(13);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID '0c0c0000-0000-0000-0000-000000000001'
\set eventCategoryID '0c0c0000-0000-0000-0000-000000000002'
\set eventID '0c0c0000-0000-0000-0000-000000000003'
\set groupCategoryID '0c0c0000-0000-0000-0000-000000000004'
\set groupID '0c0c0000-0000-0000-0000-000000000005'
\set otherAllianceID '0c0c0000-0000-0000-0000-000000000007'
\set userID '0c0c0000-0000-0000-0000-000000000006'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance owning the group and event
select fx_alliance(:'allianceID');

-- Alliance passed explicitly for a group of another alliance
select fx_alliance(:'otherAllianceID');

-- Actor whose username is snapshotted
select fx_user(:'userID', jsonb_build_object('username', 'user'));

-- Event category of the event
select fx_event_category(:'eventCategoryID', :'allianceID');

-- Group category of the group
select fx_group_category(:'groupCategoryID', :'allianceID');

-- Group whose alliance is derived when omitted
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');

-- Event scoped by the full audit row
select fx_event(:'eventID', :'groupID', :'eventCategoryID', jsonb_build_object('event_kind_id', 'virtual'));

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should derive the alliance from the group when omitted
select lives_ok(
    format($$select insert_audit_log(
        'group_team_member_added',
        %L::uuid,
        'user',
        %L::uuid,
        p_group_id => %L::uuid
    )$$, :'userID', :'userID', :'groupID'),
    'Should insert a group-scoped audit row without a alliance'
);

select is(
    (select alliance_id from audit_log where action = 'group_team_member_added'),
    :'allianceID'::uuid,
    'Should derive the alliance from the group when omitted'
);

-- Should insert an audit row with actor snapshot and default details
select lives_ok(
    format($$select insert_audit_log(
        'alliance_updated',
        %L::uuid,
        'alliance',
        %L::uuid,
        %L::uuid
    )$$, :'userID', :'allianceID', :'allianceID'),
    'Should insert an audit row'
);

select is(
    (
        select row_to_json(t.*)::jsonb - 'audit_log_id' - 'created_at'
        from (
            select *
            from audit_log
            where action = 'alliance_updated'
        ) t
    ),
    format('{
        "action": "alliance_updated",
        "actor_user_id": "%s",
        "actor_username": "user",
        "alliance_id": "%s",
        "details": {},
        "event_id": null,
        "group_id": null,
        "resource_id": "%s",
        "resource_type": "alliance"
    }', :'userID', :'allianceID', :'allianceID')::jsonb,
    'Should persist the actor snapshot and normalized details'
);

select is(
    (select created_at from audit_log where action = 'alliance_updated'),
    now(),
    'Should default the creation time to the transaction time'
);

-- Should keep an explicit alliance when given
select lives_ok(
    format($$select insert_audit_log(
        'group_team_member_removed',
        %L::uuid,
        'user',
        %L::uuid,
        %L::uuid,
        %L::uuid
    )$$, :'userID', :'userID', :'otherAllianceID', :'groupID'),
    'Should insert a group-scoped audit row with an explicit alliance'
);

select is(
    (select alliance_id from audit_log where action = 'group_team_member_removed'),
    :'otherAllianceID'::uuid,
    'Should keep an explicit alliance when given'
);

-- Should leave alliance null without a group
select lives_ok(
    format($$select insert_audit_log(
        'user_details_updated',
        %L::uuid,
        'user',
        %L::uuid
    )$$, :'userID', :'userID'),
    'Should insert an audit row without scope ids'
);

select is(
    (select alliance_id from audit_log where action = 'user_details_updated'),
    null::uuid,
    'Should leave alliance null without a group'
);

-- Should store an explicit creation time
select lives_ok(
    format($$select insert_audit_log(
        'group_updated',
        %L::uuid,
        'group',
        %L::uuid,
        %L::uuid,
        %L::uuid,
        p_created_at => '2030-01-02 03:04:05+00'::timestamptz
    )$$, :'userID', :'groupID', :'allianceID', :'groupID'),
    'Should insert an audit row with an explicit creation time'
);

select is(
    (select created_at from audit_log where action = 'group_updated'),
    '2030-01-02 03:04:05+00'::timestamptz,
    'Should store an explicit creation time'
);

-- Should store optional scope ids and explicit details
select lives_ok(
    format($$select insert_audit_log(
        'event_published',
        %L::uuid,
        'event',
        %L::uuid,
        %L::uuid,
        %L::uuid,
        %L::uuid,
        '{"subject":"Launch","recipient_count":42}'::jsonb
    )$$, :'userID', :'eventID', :'allianceID', :'groupID', :'eventID'),
    'Should insert an audit row with all scope ids'
);

select is(
    (
        select row_to_json(t.*)::jsonb - 'audit_log_id' - 'created_at'
        from (
            select *
            from audit_log
            where action = 'event_published'
        ) t
    ),
    format('{
        "action": "event_published",
        "actor_user_id": "%s",
        "actor_username": "user",
        "alliance_id": "%s",
        "details": {
            "recipient_count": 42,
            "subject": "Launch"
        },
        "event_id": "%s",
        "group_id": "%s",
        "resource_id": "%s",
        "resource_type": "event"
    }', :'userID', :'allianceID', :'eventID', :'groupID', :'eventID')::jsonb,
    'Should persist the full audit row with scope ids and explicit details'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
