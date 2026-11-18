-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(8);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID '3a3c0000-0000-0000-0000-000000000001'
\set eventCategoryID '3a3c0000-0000-0000-0000-000000000002'
\set eventID '3a3c0000-0000-0000-0000-000000000003'
\set canceledEventID '3a3c0000-0000-0000-0000-000000000004'
\set groupCategoryID '3a3c0000-0000-0000-0000-000000000005'
\set groupID '3a3c0000-0000-0000-0000-000000000006'
\set otherGroupID '3a3c0000-0000-0000-0000-000000000007'
\set proposalApprovedID '3a3c0000-0000-0000-0000-000000000008'
\set proposalRejectedID '3a3c0000-0000-0000-0000-000000000009'
\set proposalWithdrawnID '3a3c0000-0000-0000-0000-000000000010'
\set actorID '3a3c0000-0000-0000-0000-000000000011'
\set existingFeaturedSpeakerID '3a3c0000-0000-0000-0000-000000000012'
\set approvedSpeakerID '3a3c0000-0000-0000-0000-000000000013'
\set approvedCoSpeakerID '3a3c0000-0000-0000-0000-000000000014'
\set rejectedSpeakerID '3a3c0000-0000-0000-0000-000000000015'
\set withdrawnSpeakerID '3a3c0000-0000-0000-0000-000000000016'

-- ============================================================================
-- SEED DATA
-- ============================================================================

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
    'sync-speakers-alliance',
    'Sync Speakers Alliance',
    'Alliance for sync_event_approved_speakers tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Technology');

insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Meetup');

insert into "group" (group_id, alliance_id, group_category_id, name, slug) values
    (:'groupID', :'allianceID', :'groupCategoryID', 'Primary Group', 'primary-group'),
    (:'otherGroupID', :'allianceID', :'groupCategoryID', 'Other Group', 'other-group');

insert into "user" (user_id, auth_hash, email, email_verified, username, name) values
    (:'actorID', gen_random_bytes(32), 'actor@example.com', true, 'actor', 'Actor User'),
    (:'existingFeaturedSpeakerID', gen_random_bytes(32), 'featured@example.com', true, 'featured', 'Alice Featured'),
    (:'approvedSpeakerID', gen_random_bytes(32), 'approved@example.com', true, 'approved', 'Bob Approved'),
    (:'approvedCoSpeakerID', gen_random_bytes(32), 'cospeaker@example.com', true, 'cospeaker', 'Cara CoSpeaker'),
    (:'rejectedSpeakerID', gen_random_bytes(32), 'rejected@example.com', true, 'rejected', 'Dan Rejected'),
    (:'withdrawnSpeakerID', gen_random_bytes(32), 'withdrawn@example.com', true, 'withdrawn', 'Eve Withdrawn');

insert into event (
    event_id,
    group_id,
    event_category_id,
    event_kind_id,
    name,
    slug,
    description,
    timezone,
    meeting_requested,
    meeting_in_sync
) values (
    :'eventID',
    :'groupID',
    :'eventCategoryID',
    'in-person',
    'Speaker Sync Event',
    'speaker-sync-event',
    'Event for testing approved speaker synchronization',
    'UTC',
    true,
    true
);

insert into event (
    event_id,
    group_id,
    event_category_id,
    event_kind_id,
    name,
    slug,
    description,
    timezone,
    canceled
) values (
    :'canceledEventID',
    :'groupID',
    :'eventCategoryID',
    'in-person',
    'Canceled Event',
    'canceled-event',
    'Canceled event should reject speaker sync',
    'UTC',
    true
);

-- Pre-existing featured speaker who also has an approved proposal
insert into event_speaker (event_id, user_id, featured)
values (:'eventID', :'existingFeaturedSpeakerID', true);

insert into session_proposal (
    session_proposal_id,
    description,
    duration,
    session_proposal_level_id,
    title,
    user_id,
    co_speaker_user_id
) values
    (
        :'proposalApprovedID',
        'Approved talk',
        make_interval(mins => 30),
        'beginner',
        'Approved Talk',
        :'approvedSpeakerID',
        :'approvedCoSpeakerID'
    ),
    (
        :'proposalRejectedID',
        'Rejected talk',
        make_interval(mins => 30),
        'beginner',
        'Rejected Talk',
        :'rejectedSpeakerID',
        null
    ),
    (
        :'proposalWithdrawnID',
        'Withdrawn talk',
        make_interval(mins => 30),
        'beginner',
        'Withdrawn Talk',
        :'withdrawnSpeakerID',
        null
    );

insert into cfs_submission (event_id, session_proposal_id, status_id) values
    (:'eventID', :'proposalApprovedID', 'approved'),
    (:'eventID', :'proposalRejectedID', 'rejected'),
    (:'eventID', :'proposalWithdrawnID', 'withdrawn');

-- Also submit an approved proposal by the already-featured speaker to ensure featured=true is preserved
\set proposalExistingID '3a3c0000-0000-0000-0000-000000000017'
insert into session_proposal (
    session_proposal_id,
    description,
    duration,
    session_proposal_level_id,
    title,
    user_id
) values (
    :'proposalExistingID',
    'Existing featured speaker talk',
    make_interval(mins => 30),
    'beginner',
    'Existing Featured Talk',
    :'existingFeaturedSpeakerID'
);

insert into cfs_submission (event_id, session_proposal_id, status_id) values
    (:'eventID', :'proposalExistingID', 'approved');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should sync missing approved primary and co-speakers without overwriting featured status
select is(
    (
        select jsonb_build_object(
            'added_count', (result->>'added_count')::integer,
            'speakers', (
                select jsonb_agg(jsonb_build_object(
                    'user_id', speaker->>'user_id',
                    'featured', (speaker->>'featured')::boolean
                ))
                from jsonb_array_elements(result->'speakers') speaker
            )
        )
        from (
            select sync_event_approved_speakers(
                :'actorID'::uuid,
                :'groupID'::uuid,
                :'eventID'::uuid
            )::jsonb as result
        ) sub
    ),
    jsonb_build_object(
        'added_count', 2,
        'speakers', jsonb_build_array(
            jsonb_build_object('user_id', :'existingFeaturedSpeakerID', 'featured', true),
            jsonb_build_object('user_id', :'approvedSpeakerID', 'featured', false),
            jsonb_build_object('user_id', :'approvedCoSpeakerID', 'featured', false)
        )
    ),
    'Should insert approved primary and co-speakers and preserve featured status'
);

-- Should mark automatic meetings out of sync when speakers are inserted
select is(
    (select meeting_in_sync from event where event_id = :'eventID'),
    false,
    'Should mark automatic meeting out of sync when approved speakers are added'
);

-- Should record audit log when speakers are added
select is(
    (
        select count(*)::integer
        from audit_log
        where resource_type = 'event'
        and resource_id = :'eventID'
        and action = 'event_updated'
        and actor_user_id = :'actorID'
    ),
    1,
    'Should insert an audit log entry when speakers are added'
);

-- Should be idempotent when called again with no new approved speakers
select is(
    (
        select (sync_event_approved_speakers(
            :'actorID'::uuid,
            :'groupID'::uuid,
            :'eventID'::uuid
        )->>'added_count')::integer
    ),
    0,
    'Should return added_count = 0 on subsequent sync'
);

-- Should not create duplicate audit log entries on no-op sync
select is(
    (
        select count(*)::integer
        from audit_log
        where resource_type = 'event'
        and resource_id = :'eventID'
        and action = 'event_updated'
    ),
    1,
    'Should not insert an audit log entry when no speakers were added'
);

-- Should not add rejected or withdrawn speakers
select is(
    (
        select count(*)::integer
        from event_speaker
        where event_id = :'eventID'
        and user_id in (:'rejectedSpeakerID', :'withdrawnSpeakerID')
    ),
    0,
    'Should exclude rejected and withdrawn submission speakers'
);

-- Should reject events belonging to another group
select throws_ok(
    format(
        $$select sync_event_approved_speakers('%s'::uuid, '%s'::uuid, '%s'::uuid)$$,
        :'actorID',
        :'otherGroupID',
        :'eventID'
    ),
    'event not found',
    'Should fail when group does not own event'
);

-- Should reject canceled events
select throws_ok(
    format(
        $$select sync_event_approved_speakers('%s'::uuid, '%s'::uuid, '%s'::uuid)$$,
        :'actorID',
        :'groupID',
        :'canceledEventID'
    ),
    'event not found',
    'Should fail for canceled events'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
