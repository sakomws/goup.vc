-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID '4b2d0000-0000-0000-0000-000000000001'
\set eventCategoryID '4b2d0000-0000-0000-0000-000000000002'
\set groupCategoryID '4b2d0000-0000-0000-0000-000000000003'
\set groupID '4b2d0000-0000-0000-0000-000000000004'
\set eventID '4b2d0000-0000-0000-0000-000000000005'
\set proposalID '4b2d0000-0000-0000-0000-000000000006'
\set submissionID '4b2d0000-0000-0000-0000-000000000007'
\set sessionID '4b2d0000-0000-0000-0000-000000000008'
\set labelID '4b2d0000-0000-0000-0000-000000000009'
\set speakerID '4b2d0000-0000-0000-0000-000000000010'
\set organizerID '4b2d0000-0000-0000-0000-000000000011'

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
    'speaker-prep-alliance',
    'Speaker Prep Alliance',
    'Alliance used for speaker prep email tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Tech');
insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Meetup');

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values (:'groupID', :'allianceID', :'groupCategoryID', 'Speaker Prep Group', 'speaker-prep-group');

insert into "user" (user_id, auth_hash, email, username, email_verified) values
    (:'speakerID', gen_random_bytes(32), 'speaker@example.com', 'prep-speaker', true),
    (:'organizerID', gen_random_bytes(32), 'organizer@example.com', 'prep-organizer', true);

insert into session_proposal (
    session_proposal_id,
    description,
    duration,
    session_proposal_level_id,
    title,
    user_id
) values (
    :'proposalID',
    'Approved talk',
    make_interval(mins => 30),
    'beginner',
    'Opening keynote',
    :'speakerID'
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
    published,
    venue_name
) values (
    :'eventID',
    :'eventCategoryID',
    'in-person',
    :'groupID',
    'Speaker Prep Event',
    'speaker-prep-event',
    'Event used for speaker prep email tests',
    'UTC',
    true,
    'Baku Hall'
);

insert into cfs_submission (
    cfs_submission_id,
    event_id,
    session_proposal_id,
    status_id
) values (
    :'submissionID',
    :'eventID',
    :'proposalID',
    'approved'
);

insert into session (
    session_id,
    cfs_submission_id,
    event_id,
    name,
    session_kind_id,
    starts_at
) values (
    :'sessionID',
    :'submissionID',
    :'eventID',
    'Opening keynote',
    'in-person',
    '2024-02-01 10:00:00+00'
);

insert into event_cfs_label (event_cfs_label_id, event_id, name, color)
values (:'labelID', :'eventID', 'AI', '#DBEAFE');
insert into cfs_submission_label (cfs_submission_id, event_cfs_label_id)
values (:'submissionID', :'labelID');

-- ============================================================================
-- TESTS
-- ============================================================================

select is(
    get_cfs_submission_prep_email_context(
        :'groupID'::uuid,
        :'eventID'::uuid,
        :'submissionID'::uuid
    )::jsonb,
    jsonb_build_object(
        'linked_session_id', :'sessionID'::uuid,
        'prep_emails', '[]'::jsonb,
        'session_starts_at', floor(extract(epoch from timestamptz '2024-02-01 10:00:00+00'))::bigint,
        'speaker_user_id', :'speakerID'::uuid,
        'status_id', 'approved',
        'title', 'Opening keynote',
        'tracks', jsonb_build_array('AI'),
        'venue_name', 'Baku Hall'
    ),
    'Should describe the schedule facts before any prep email is sent'
);

select lives_ok(
    format(
        $$select send_cfs_submission_prep_email(
            %L::uuid,
            %L::uuid,
            %L::uuid,
            %L::uuid,
            'schedule',
            '{"subject":"Your time slot"}'::jsonb,
            'Your time slot',
            'The event page has the schedule.'
        )$$,
        :'organizerID',
        :'groupID',
        :'eventID',
        :'submissionID'
    ),
    'Should send and record the schedule email'
);

select is(
    (
        select prep_step
        from cfs_submission_prep_email
        where cfs_submission_id = :'submissionID'::uuid
    ),
    'schedule',
    'Should record the schedule email on the submission'
);

select throws_ok(
    format(
        $$select send_cfs_submission_prep_email(
            %L::uuid,
            %L::uuid,
            %L::uuid,
            %L::uuid,
            'schedule',
            '{"subject":"Your time slot"}'::jsonb,
            'Your time slot',
            'The event page has the schedule.'
        )$$,
        :'organizerID',
        :'groupID',
        :'eventID',
        :'submissionID'
    ),
    'OCG01',
    'This speaker prep email was already sent.',
    'Should not send the same speaker prep email twice'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
