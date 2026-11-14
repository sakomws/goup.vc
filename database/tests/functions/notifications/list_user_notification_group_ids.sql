-- Tests listing groups available for user notification mutes.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set acceptedTeamGroupID '9e020000-0000-0000-0000-000000000001'
\set attendeeEventID '9e020000-0000-0000-0000-000000000002'
\set attendeeGroupID '9e020000-0000-0000-0000-000000000003'
\set cohostApprovedGroupID '9e020000-0000-0000-0000-000000000004'
\set cohostCanceledGroupID '9e020000-0000-0000-0000-000000000005'
\set cohostPendingGroupID '9e020000-0000-0000-0000-000000000006'
\set cohostRejectedGroupID '9e020000-0000-0000-0000-000000000007'
\set allianceID '9e020000-0000-0000-0000-000000000008'
\set deletedEventGroupID '9e020000-0000-0000-0000-000000000009'
\set deletedEventID '9e020000-0000-0000-0000-000000000010'
\set deletedGroupID '9e020000-0000-0000-0000-000000000011'
\set deletedGroupMemberUserID '9e020000-0000-0000-0000-000000000012'
\set eventCategoryID '9e020000-0000-0000-0000-000000000013'
\set eventSpeakerEventID '9e020000-0000-0000-0000-000000000014'
\set eventSpeakerGroupID '9e020000-0000-0000-0000-000000000015'
\set groupCategoryID '9e020000-0000-0000-0000-000000000016'
\set inactiveAllianceGroupCategoryID '9e020000-0000-0000-0000-000000000017'
\set inactiveAllianceGroupID '9e020000-0000-0000-0000-000000000018'
\set inactiveAllianceID '9e020000-0000-0000-0000-000000000019'
\set inactiveGroupID '9e020000-0000-0000-0000-000000000020'
\set memberGroupID '9e020000-0000-0000-0000-000000000021'
\set pendingTeamGroupID '9e020000-0000-0000-0000-000000000022'
\set sessionID '9e020000-0000-0000-0000-000000000023'
\set sessionSpeakerEventID '9e020000-0000-0000-0000-000000000024'
\set sessionSpeakerGroupID '9e020000-0000-0000-0000-000000000025'
\set userID '9e020000-0000-0000-0000-000000000027'
\set waitlistEventID '9e020000-0000-0000-0000-000000000028'
\set waitlistGroupID '9e020000-0000-0000-0000-000000000029'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Active alliance containing available groups
select fx_alliance(:'allianceID');

-- Inactive alliance containing an excluded group
select fx_alliance(:'inactiveAllianceID', jsonb_build_object('active', false));

-- Event category used by event-connected groups
select fx_event_category(:'eventCategoryID', :'allianceID');

-- Group category used by active-alliance groups
select fx_group_category(:'groupCategoryID', :'allianceID');

-- Group category used by the inactive-alliance group
select fx_group_category(:'inactiveAllianceGroupCategoryID', :'inactiveAllianceID');

-- User whose connected groups are listed
select fx_user(:'userID');

-- Separate user used by the deleted-group exclusion scenario
select fx_user(:'deletedGroupMemberUserID');

-- Group connected through accepted team membership
select fx_group(:'acceptedTeamGroupID', :'allianceID', :'groupCategoryID');

-- Group connected through an attendee row
select fx_group(:'attendeeGroupID', :'allianceID', :'groupCategoryID');

-- Group connected through approved co-hosting of an attended event
select fx_group(:'cohostApprovedGroupID', :'allianceID', :'groupCategoryID');

-- Canceled co-host group excluded from the result
select fx_group(:'cohostCanceledGroupID', :'allianceID', :'groupCategoryID');

-- Pending co-host group excluded from the result
select fx_group(:'cohostPendingGroupID', :'allianceID', :'groupCategoryID');

-- Rejected co-host group excluded from the result
select fx_group(:'cohostRejectedGroupID', :'allianceID', :'groupCategoryID');

-- Group connected only through a deleted event
select fx_group(:'deletedEventGroupID', :'allianceID', :'groupCategoryID');

-- Deleted group excluded even when a user belongs to it
select fx_group(
    :'deletedGroupID',
    :'allianceID',
    :'groupCategoryID',
    jsonb_build_object(
        'active', false,
        'deleted', true,
        'deleted_at', '2024-01-01 00:00:00+00'
    )
);

-- Group connected through event speaking
select fx_group(:'eventSpeakerGroupID', :'allianceID', :'groupCategoryID');

-- Group excluded because its alliance is inactive
select fx_group(:'inactiveAllianceGroupID', :'inactiveAllianceID', :'inactiveAllianceGroupCategoryID');

-- Inactive group excluded even when a user belongs to it
select fx_group(
    :'inactiveGroupID',
    :'allianceID',
    :'groupCategoryID',
    jsonb_build_object('active', false)
);

-- Group connected through membership
select fx_group(:'memberGroupID', :'allianceID', :'groupCategoryID');

-- Pending team group excluded from the result
select fx_group(:'pendingTeamGroupID', :'allianceID', :'groupCategoryID');

-- Group connected through session speaking
select fx_group(:'sessionSpeakerGroupID', :'allianceID', :'groupCategoryID');

-- Group connected through an event waitlist row
select fx_group(:'waitlistGroupID', :'allianceID', :'groupCategoryID');

-- Attended event with approved and unavailable co-host states
select fx_event(:'attendeeEventID', :'attendeeGroupID', :'eventCategoryID');

-- Deleted event whose group should not be returned
select fx_event(:'deletedEventID', :'deletedEventGroupID', :'eventCategoryID', jsonb_build_object('deleted', true));

-- Event connected through event speaker
select fx_event(:'eventSpeakerEventID', :'eventSpeakerGroupID', :'eventCategoryID');

-- Event connected through session speaker
select fx_event(:'sessionSpeakerEventID', :'sessionSpeakerGroupID', :'eventCategoryID');

-- Event connected through waitlist
select fx_event(:'waitlistEventID', :'waitlistGroupID', :'eventCategoryID');

-- Accepted team membership connection
insert into group_team (group_id, user_id, accepted, role)
values (:'acceptedTeamGroupID', :'userID', true, 'admin');

-- Pending team membership excluded from connections
insert into group_team (group_id, user_id, accepted, role)
values (:'pendingTeamGroupID', :'userID', false, 'admin');

-- Direct group membership connection
insert into group_member (group_id, user_id)
values (:'memberGroupID', :'userID');

-- Deleted group membership excluded from the result
insert into group_member (group_id, user_id)
values (:'deletedGroupID', :'deletedGroupMemberUserID');

-- Inactive alliance group membership excluded from the result
insert into group_member (group_id, user_id)
values (:'inactiveAllianceGroupID', :'userID');

-- Inactive group membership excluded from the result
insert into group_member (group_id, user_id)
values (:'inactiveGroupID', :'userID');

-- Attendee connection to the event owner and approved co-host groups
insert into event_attendee (event_id, user_id, status)
values (:'attendeeEventID', :'userID', 'confirmed');

-- Attendee row on a deleted event excluded from connections
insert into event_attendee (event_id, user_id, status)
values (:'deletedEventID', :'userID', 'confirmed');

-- Waitlist connection to an event owner group
insert into event_waitlist (event_id, user_id)
values (:'waitlistEventID', :'userID');

-- Event speaker connection to an event owner group
insert into event_speaker (event_id, user_id)
values (:'eventSpeakerEventID', :'userID');

-- Session row for the session speaker connection
insert into session (event_id, name, session_id, session_kind_id, starts_at)
values (:'sessionSpeakerEventID', 'Notification Preferences', :'sessionID', 'in-person', '2030-01-01 10:00:00+00');

-- Session speaker connection to an event owner group
insert into session_speaker (session_id, user_id)
values (:'sessionID', :'userID');

-- Approved co-host group connected through the attended event
insert into event_cohost (
    cohost_group_id,
    decided_at,
    decided_by_user_id,
    event_id,
    requested_by_user_id,
    status
)
values (
    :'cohostApprovedGroupID',
    '2025-01-01 00:00:00+00',
    :'userID',
    :'attendeeEventID',
    :'userID',
    'approved'
);

-- Revoked co-host group excluded from connections
insert into event_cohost (
    cohost_group_id,
    decided_at,
    decided_by_user_id,
    event_id,
    requested_by_user_id,
    status
)
values (
    :'cohostCanceledGroupID',
    '2025-01-01 00:00:00+00',
    :'userID',
    :'attendeeEventID',
    :'userID',
    'revoked'
);

-- Pending co-host group excluded from connections
insert into event_cohost (cohost_group_id, event_id, requested_by_user_id, status)
values (:'cohostPendingGroupID', :'attendeeEventID', :'userID', 'pending');

-- Rejected co-host group excluded from connections
insert into event_cohost (
    cohost_group_id,
    decided_at,
    decided_by_user_id,
    event_id,
    requested_by_user_id,
    status
)
values (
    :'cohostRejectedGroupID',
    '2025-01-01 00:00:00+00',
    :'userID',
    :'attendeeEventID',
    :'userID',
    'rejected'
);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return every available connected group once
select is(
    list_user_notification_group_ids(:'userID'),
    array[
        :'acceptedTeamGroupID',
        :'attendeeGroupID',
        :'cohostApprovedGroupID',
        :'eventSpeakerGroupID',
        :'memberGroupID',
        :'sessionSpeakerGroupID',
        :'waitlistGroupID'
    ]::uuid[],
    'Should return every available connected group once'
);

-- Should return no unavailable connected groups
select is(
    list_user_notification_group_ids(:'deletedGroupMemberUserID'),
    '{}'::uuid[],
    'Should return no unavailable connected groups'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
