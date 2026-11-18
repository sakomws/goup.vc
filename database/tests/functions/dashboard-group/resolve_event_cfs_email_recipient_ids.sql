-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(5);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID '3a1c0000-0000-0000-0000-000000000001'
\set eventCategoryID '3a1c0000-0000-0000-0000-000000000002'
\set groupCategoryID '3a1c0000-0000-0000-0000-000000000003'
\set groupID '3a1c0000-0000-0000-0000-000000000004'
\set otherGroupID '3a1c0000-0000-0000-0000-000000000005'
\set eventID '3a1c0000-0000-0000-0000-000000000006'
\set otherEventID '3a1c0000-0000-0000-0000-000000000007'
\set approvedProposalID '3a1c0000-0000-0000-0000-000000000008'
\set rejectedProposalID '3a1c0000-0000-0000-0000-000000000009'
\set activeProposalID '3a1c0000-0000-0000-0000-000000000010'
\set withdrawnProposalID '3a1c0000-0000-0000-0000-000000000011'
\set unverifiedProposalID '3a1c0000-0000-0000-0000-000000000012'
\set optedOutProposalID '3a1c0000-0000-0000-0000-000000000013'
\set otherProposalID '3a1c0000-0000-0000-0000-000000000014'
\set approvedSubmissionID '3a1c0000-0000-0000-0000-000000000015'
\set rejectedSubmissionID '3a1c0000-0000-0000-0000-000000000016'
\set activeSubmissionID '3a1c0000-0000-0000-0000-000000000017'
\set withdrawnSubmissionID '3a1c0000-0000-0000-0000-000000000018'
\set unverifiedSubmissionID '3a1c0000-0000-0000-0000-000000000019'
\set optedOutSubmissionID '3a1c0000-0000-0000-0000-000000000020'
\set otherSubmissionID '3a1c0000-0000-0000-0000-000000000021'
\set approvedUserID '3a1c0000-0000-0000-0000-000000000030'
\set coSpeakerUserID '3a1c0000-0000-0000-0000-000000000031'
\set rejectedUserID '3a1c0000-0000-0000-0000-000000000032'
\set activeUserID '3a1c0000-0000-0000-0000-000000000033'
\set withdrawnUserID '3a1c0000-0000-0000-0000-000000000034'
\set unverifiedUserID '3a1c0000-0000-0000-0000-000000000035'
\set optedOutUserID '3a1c0000-0000-0000-0000-000000000036'
\set otherUserID '3a1c0000-0000-0000-0000-000000000037'

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
    'cfs-email-alliance',
    'CFS Email Alliance',
    'Alliance used for submission email recipient tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Tech');
insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Meetup');

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values
    (:'groupID', :'allianceID', :'groupCategoryID', 'CFS Email Group', 'cfs-email-group'),
    (:'otherGroupID', :'allianceID', :'groupCategoryID', 'Other Group', 'cfs-email-other-group');

insert into "user" (user_id, auth_hash, email, username, email_verified) values
    (:'approvedUserID', gen_random_bytes(32), 'approved@example.com', 'approved-speaker', true),
    (:'coSpeakerUserID', gen_random_bytes(32), 'cospeaker@example.com', 'approved-cospeaker', true),
    (:'rejectedUserID', gen_random_bytes(32), 'rejected@example.com', 'rejected-speaker', true),
    (:'activeUserID', gen_random_bytes(32), 'active@example.com', 'active-speaker', true),
    (:'withdrawnUserID', gen_random_bytes(32), 'withdrawn@example.com', 'withdrawn-speaker', true),
    (:'unverifiedUserID', gen_random_bytes(32), 'unverified@example.com', 'unverified-speaker', false),
    (:'optedOutUserID', gen_random_bytes(32), 'opted-out@example.com', 'opted-out-speaker', true),
    (:'otherUserID', gen_random_bytes(32), 'other@example.com', 'other-speaker', true);

insert into user_notification_opt_out (notification_category_id, user_id)
values ('organizer-messages', :'optedOutUserID');

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
        :'approvedProposalID',
        'Approved talk',
        make_interval(mins => 45),
        'beginner',
        'Approved talk',
        :'approvedUserID',
        :'coSpeakerUserID'
    ),
    (
        :'rejectedProposalID',
        'Rejected talk',
        make_interval(mins => 30),
        'beginner',
        'Rejected talk',
        :'rejectedUserID',
        null
    ),
    (
        :'activeProposalID',
        'Active talk',
        make_interval(mins => 30),
        'beginner',
        'Active talk',
        :'activeUserID',
        null
    ),
    (
        :'withdrawnProposalID',
        'Withdrawn talk',
        make_interval(mins => 30),
        'beginner',
        'Withdrawn talk',
        :'withdrawnUserID',
        null
    ),
    (
        :'unverifiedProposalID',
        'Unverified talk',
        make_interval(mins => 30),
        'beginner',
        'Unverified talk',
        :'unverifiedUserID',
        null
    ),
    (
        :'optedOutProposalID',
        'Opted out talk',
        make_interval(mins => 30),
        'beginner',
        'Opted out talk',
        :'optedOutUserID',
        null
    ),
    (
        :'otherProposalID',
        'Other group talk',
        make_interval(mins => 30),
        'beginner',
        'Other group talk',
        :'otherUserID',
        null
    );

insert into event (
    event_id,
    event_category_id,
    event_kind_id,
    group_id,
    name,
    slug,
    description,
    timezone,
    published
) values (
    :'eventID',
    :'eventCategoryID',
    'in-person',
    :'groupID',
    'CFS Email Event',
    'cfs-email-event',
    'Event used for submission email recipient tests',
    'UTC',
    true
), (
    :'otherEventID',
    :'eventCategoryID',
    'in-person',
    :'otherGroupID',
    'Other CFS Email Event',
    'cfs-email-other-event',
    'Event in another group',
    'UTC',
    true
);

insert into cfs_submission (
    cfs_submission_id,
    event_id,
    session_proposal_id,
    status_id
) values
    (:'approvedSubmissionID', :'eventID', :'approvedProposalID', 'approved'),
    (:'rejectedSubmissionID', :'eventID', :'rejectedProposalID', 'rejected'),
    (:'activeSubmissionID', :'eventID', :'activeProposalID', 'not-reviewed'),
    (:'withdrawnSubmissionID', :'eventID', :'withdrawnProposalID', 'withdrawn'),
    (:'unverifiedSubmissionID', :'eventID', :'unverifiedProposalID', 'approved'),
    (:'optedOutSubmissionID', :'eventID', :'optedOutProposalID', 'approved'),
    (:'otherSubmissionID', :'otherEventID', :'otherProposalID', 'approved');

-- ============================================================================
-- TESTS
-- ============================================================================

select is(
    resolve_event_cfs_email_recipient_ids(:'groupID'::uuid, :'eventID'::uuid, 'approved'),
    array[:'approvedUserID'::uuid, :'coSpeakerUserID'::uuid],
    'Should email approved speakers and co-speakers who accept organizer email'
);

select is(
    resolve_event_cfs_email_recipient_ids(:'groupID'::uuid, :'eventID'::uuid, 'rejected'),
    array[:'rejectedUserID'::uuid],
    'Should email rejected speakers who accept organizer email'
);

select is(
    resolve_event_cfs_email_recipient_ids(:'groupID'::uuid, :'eventID'::uuid, 'all'),
    array[
        :'approvedUserID'::uuid,
        :'coSpeakerUserID'::uuid,
        :'rejectedUserID'::uuid,
        :'activeUserID'::uuid
    ],
    'Should email every active speaker and skip withdrawn, unverified, and opted-out speakers'
);

select is(
    resolve_event_cfs_email_recipient_ids(:'otherGroupID'::uuid, :'eventID'::uuid, 'all'),
    array[]::uuid[],
    'Should return no recipients when the event belongs to another group'
);

select is(
    resolve_event_cfs_email_recipient_ids(:'groupID'::uuid, :'eventID'::uuid, 'unknown'),
    array[]::uuid[],
    'Should return no recipients for an unknown audience'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
