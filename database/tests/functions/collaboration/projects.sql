begin;
select plan(46);

\set allianceID 'ca110000-0000-0000-0000-000000000001'
\set groupCategoryID 'ca110000-0000-0000-0000-000000000002'
\set groupID 'ca110000-0000-0000-0000-000000000003'
\set ownerID 'ca110000-0000-0000-0000-000000000004'
\set contributorID 'ca110000-0000-0000-0000-000000000005'
\set otherMemberID 'ca110000-0000-0000-0000-000000000006'
\set outsiderID 'ca110000-0000-0000-0000-000000000007'
\set sessionID 'ca110000-0000-0000-0000-000000000008'

insert into alliance (
    alliance_id, name, display_name, description, banner_mobile_url,
    banner_url, logo_url
) values (
    :'allianceID', 'collaboration-tests', 'Collaboration Tests',
    'Collaboration function tests', 'https://example.com/mobile.png',
    'https://example.com/banner.png', 'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Collaboration');

insert into "group" (
    group_id, alliance_id, group_category_id, name, description, slug
) values (
    :'groupID', :'allianceID', :'groupCategoryID',
    'Collaboration Group', 'Collaboration test group', 'collaboration-group'
);

insert into "user" (
    user_id, name, auth_hash, email, email_verified, username
) values
    (:'ownerID', 'Project Owner', gen_random_bytes(32), 'collaboration-owner@example.com', true, 'collaboration-owner'),
    (:'contributorID', 'Project Contributor', gen_random_bytes(32), 'collaboration-contributor@example.com', true, 'collaboration-contributor'),
    (:'otherMemberID', 'Other Project Member', gen_random_bytes(32), 'collaboration-member@example.com', true, 'collaboration-member'),
    (:'outsiderID', 'Project Outsider', gen_random_bytes(32), 'collaboration-outsider@example.com', true, 'collaboration-outsider');

insert into group_team (accepted, group_id, role, user_id)
values (true, :'groupID', 'admin', :'ownerID');

select has_table('collaboration_project');
select has_table('collaboration_project_member');
select has_table('collaboration_project_goal');
select has_table('collaboration_project_task');
select has_table('collaboration_project_update');
select has_table('collaboration_project_activity');
select has_table('collaboration_project_outcome');
select has_table('collaboration_project_evidence');
select has_table('collaboration_office_hour_session');
select has_table('collaboration_office_hour_booking');
select has_table('collaboration_notification_delivery');

select has_function('collaboration_project_json', array['collaboration_project']);
select has_function('get_collaboration_outcome_summary', array['uuid']);
select has_function('create_collaboration_project', array['uuid', 'uuid', 'jsonb']);
select has_function('get_group_collaboration_dashboard', array['uuid', 'uuid']);
select has_function('get_public_collaboration_project', array['uuid', 'text', 'text']);
select has_function('submit_collaboration_update', array['uuid', 'uuid', 'text', 'text', 'text']);
select has_function('book_collaboration_office_hour', array['uuid', 'uuid', 'text']);
select has_function('enqueue_due_collaboration_reminders', array['timestamp with time zone']);

select results_eq(
    $$select display_name from group_permission where group_permission_id = 'group.projects.write'$$,
    $$values ('Projects Write'::text)$$,
    'project management permission is registered'
);

select throws_ok(
    format(
        'select create_collaboration_project(%L, %L, %L::jsonb)',
        :'outsiderID',
        :'groupID',
        '{"slug":"forbidden","name":"Forbidden","summary":"Forbidden"}'
    ),
    'project management permission required',
    'project creation enforces group project permission in the database'
);

select create_collaboration_project(
    :'ownerID',
    :'groupID',
    jsonb_build_object(
        'slug', 'public-project',
        'name', 'Public Project',
        'summary', 'A public collaboration project',
        'lifecycle', 'active',
        'visibility', 'public'
    )
) as project_id \gset

select ok(
    exists (
        select 1
        from collaboration_project_member
        where collaboration_project_id = :'project_id'
          and user_id = :'ownerID'
          and role = 'owner'
          and invitation_status = 'accepted'
    ),
    'project creator becomes its accepted owner'
);

select throws_ok(
    format(
        'select invite_collaboration_project_member(%L, %L, %L, %L, %L)',
        :'ownerID', :'groupID', :'project_id', :'ownerID', 'viewer'
    ),
    'project owner role cannot be changed by invitation',
    'the invitation flow cannot demote the project owner'
);

select create_collaboration_project(
    :'ownerID',
    :'groupID',
    jsonb_build_object(
        'slug', 'completed-project',
        'name', 'Completed Project',
        'summary', 'A completed collaboration project',
        'lifecycle', 'completed',
        'visibility', 'members'
    )
) as completed_project_id \gset

select ok(
    (select completed_at is not null from collaboration_project where collaboration_project_id = :'completed_project_id'),
    'creating a completed project records its completion time'
);

select throws_ok(
    format(
        'select invite_collaboration_project_member(%L, %L, %L, %L, %L)',
        :'outsiderID', :'groupID', :'project_id', :'contributorID', 'contributor'
    ),
    'project management permission required',
    'member invitations enforce group project permission in the database'
);

select invite_collaboration_project_member(
    :'ownerID', :'groupID', :'project_id', :'contributorID', 'contributor'
) as member_id \gset

select invite_collaboration_project_member(
    :'ownerID', :'groupID', :'project_id', :'contributorID', 'contributor'
) as repeated_member_id \gset

select is(
    :'repeated_member_id'::uuid,
    :'member_id'::uuid,
    'repeating an unchanged pending invitation returns the same membership'
);

select is(
    (
        select count(*)
        from collaboration_notification_delivery
        where recipient_user_id = :'contributorID'
          and kind = 'invitation'
    ),
    1::bigint,
    'repeating an unchanged invitation does not enqueue another delivery'
);

select is(
    (
        select count(*)
        from collaboration_project_activity
        where subject_id = :'member_id'
          and kind = 'member.invited'
    ),
    1::bigint,
    'repeating an unchanged invitation does not duplicate activity'
);

select throws_ok(
    format(
        'select submit_collaboration_update(%L, %L, %L, null, null)',
        :'contributorID', :'project_id', 'Pending members cannot post'
    ),
    'accepted owner or contributor membership required',
    'pending members cannot post updates'
);

update collaboration_project_member
set invitation_status = 'accepted', responded_at = current_timestamp
where collaboration_project_member_id = :'member_id';

select submit_collaboration_update(
    :'contributorID',
    :'project_id',
    '  Progress is on track.  ',
    '  Internal blocker  ',
    '  Ship the next milestone  '
) as update_id \gset

select ok(
    :'update_id'::uuid is not null,
    'an accepted contributor can post an update'
);

select results_eq(
    format(
        'select body, blockers, next_steps from collaboration_project_update where collaboration_project_update_id = %L',
        :'update_id'
    ),
    $$values ('Progress is on track.'::text, 'Internal blocker'::text, 'Ship the next milestone'::text)$$,
    'update text is normalized before storage'
);

select ok(
    get_public_collaboration_project(:'allianceID', 'collaboration-group', 'public-project') is not null,
    'public active projects resolve through the public lookup'
);

select ok(
    not (
        (
            get_public_collaboration_project(:'allianceID', 'collaboration-group', 'public-project')
            #> '{updates,0}'
        ) ? 'blockers'
    ),
    'public project payloads do not expose private blocker notes'
);

update collaboration_project
set lifecycle = 'archived'
where collaboration_project_id = :'project_id';

select throws_ok(
    format(
        'select submit_collaboration_update(%L, %L, %L, null, null)',
        :'contributorID', :'project_id', 'Archived projects are closed'
    ),
    'accepted owner or contributor membership required',
    'archived projects reject new updates'
);

update collaboration_project
set lifecycle = 'active'
where collaboration_project_id = :'project_id';

select throws_ok(
    format(
        'insert into collaboration_project_update (collaboration_project_id, collaboration_project_member_id, body) values (%L, %L, %L)',
        :'completed_project_id', :'member_id', 'Cross-project update'
    ),
    'update author must be an accepted owner or contributor of the same project',
    'updates cannot reference a member from another project'
);

select throws_ok(
    format(
        'insert into collaboration_project_task (collaboration_project_id, created_by, title, status) values (%L, %L, %L, %L)',
        :'project_id', :'ownerID', 'Invalid done task', 'done'
    ),
    '23514',
    null,
    'done tasks require a completion timestamp'
);

insert into collaboration_office_hour_session (
    collaboration_office_hour_session_id, collaboration_project_id,
    expert_user_id, created_by, title, starts_at, ends_at, capacity
) values (
    :'sessionID', :'project_id', :'ownerID', :'ownerID', 'Architecture review',
    current_timestamp + interval '1 day',
    current_timestamp + interval '1 day 1 hour',
    1
);

select throws_ok(
    format(
        'select book_collaboration_office_hour(%L, %L, null)',
        :'outsiderID', :'sessionID'
    ),
    'bookable session or accepted membership not found',
    'office hours require accepted project membership'
);

select book_collaboration_office_hour(
    :'contributorID', :'sessionID', '  Review the data model  '
) as booking_id \gset

select ok(
    :'booking_id'::uuid is not null,
    'an accepted project member can book office hours'
);

select book_collaboration_office_hour(
    :'contributorID', :'sessionID', 'Review the data model'
) as repeated_booking_id \gset

select is(
    :'repeated_booking_id'::uuid,
    :'booking_id'::uuid,
    'repeating an active booking returns the same booking'
);

select is(
    (
        select count(*)
        from collaboration_project_activity
        where subject_id = :'booking_id'
          and kind = 'office_hour.booked'
    ),
    1::bigint,
    'repeating a booking does not duplicate activity'
);

select is(
    (
        select count(*)
        from collaboration_notification_delivery
        where recipient_user_id = :'contributorID'
          and kind = 'booking'
    ),
    1::bigint,
    'repeating a booking does not enqueue another confirmation'
);

insert into collaboration_project_member (
    collaboration_project_id, user_id, role, invitation_status,
    invited_by, responded_at
) values (
    :'project_id', :'otherMemberID', 'contributor', 'accepted',
    :'ownerID', current_timestamp
);

select throws_ok(
    format(
        'select book_collaboration_office_hour(%L, %L, null)',
        :'otherMemberID', :'sessionID'
    ),
    'office-hour session is at capacity',
    'office-hour capacity is enforced'
);

select is(
    enqueue_due_collaboration_reminders(
        (select starts_at - interval '12 hours' from collaboration_office_hour_session where collaboration_office_hour_session_id = :'sessionID')
    ),
    1,
    'the first due reminder is enqueued'
);

select is(
    enqueue_due_collaboration_reminders(
        (select starts_at - interval '12 hours' from collaboration_office_hour_session where collaboration_office_hour_session_id = :'sessionID')
    ),
    0,
    'repeating reminder scheduling is idempotent'
);

select ok(
    not (
        select payload ?| array['question', 'meeting_url']
        from collaboration_notification_delivery
        where recipient_user_id = :'contributorID'
          and kind = 'session_reminder'
    ),
    'reminder payloads omit private questions and meeting URLs'
);

select results_eq(
    format(
        $$select
            (session->>'available_capacity')::bigint,
            (session->>'can_book')::boolean
          from jsonb_array_elements(
            get_group_collaboration_dashboard(%L, %L)->'sessions'
          ) session
          where session->>'collaboration_office_hour_session_id' = %L$$,
        :'groupID', :'contributorID', :'sessionID'
    ),
    $$values (0::bigint, false)$$,
    'dashboard sessions expose remaining capacity and booking eligibility'
);

select * from finish();
rollback;
