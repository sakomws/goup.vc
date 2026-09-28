begin;
select plan(24);

\set allianceID '5e160000-0000-0000-0000-000000000001'
\set groupCategoryID '5e160000-0000-0000-0000-000000000002'
\set eventCategoryID '5e160000-0000-0000-0000-000000000003'
\set groupID '5e160000-0000-0000-0000-000000000004'
\set sponsorID '5e160000-0000-0000-0000-000000000005'
\set eventID '5e160000-0000-0000-0000-000000000006'
\set otherEventID '5e160000-0000-0000-0000-000000000007'
\set zeroEventID '5e160000-0000-0000-0000-000000000008'
\set userOneID '5e160000-0000-0000-0000-000000000009'
\set userTwoID '5e160000-0000-0000-0000-00000000000a'
\set userThreeID '5e160000-0000-0000-0000-00000000000b'
\set targetLeadID '5e160000-0000-0000-0000-00000000000c'
\set otherLeadID '5e160000-0000-0000-0000-00000000000d'
\set shareToken '5e160000-0000-0000-0000-00000000000e'

select has_table('event_sponsor_engagement_daily');
select has_table('event_sponsor_manual_engagement');
select has_table('event_sponsor_report_share');
select has_pk('event_sponsor_engagement_daily');
select has_pk('event_sponsor_manual_engagement');
select has_pk('event_sponsor_report_share');
select fk_ok(
    'event_sponsor_engagement_daily',
    array['group_sponsor_id', 'event_id']::name[],
    'event_sponsor',
    array['group_sponsor_id', 'event_id']::name[]
);
select fk_ok(
    'event_sponsor_manual_engagement',
    array['group_sponsor_id', 'event_id']::name[],
    'event_sponsor',
    array['group_sponsor_id', 'event_id']::name[]
);
select has_function(
    'record_event_sponsor_engagement',
    array['uuid', 'uuid', 'text', 'uuid']::name[]
);
select has_function('get_event_sponsor_report', array['uuid']::name[]);
select has_function('get_public_event_sponsor_report', array['uuid']::name[]);
select has_function(
    'update_event_sponsor_manual_engagement',
    array['uuid', 'uuid', 'uuid', 'uuid', 'jsonb']::name[]
);
select has_function(
    'create_event_sponsor_report_share',
    array['uuid', 'uuid', 'uuid']::name[]
);
select has_function(
    'revoke_event_sponsor_report_share',
    array['uuid', 'uuid']::name[]
);
select col_is_unique('event_sponsor_report_share', 'token');

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
    'sponsor-report-alliance',
    'Sponsor Report Alliance',
    'Alliance for sponsor report tests',
    'https://example.com/mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Sponsor report groups');

insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Sponsor report events');

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values (:'groupID', :'allianceID', :'groupCategoryID', 'Report Group', 'report-group');

insert into "user" (user_id, auth_hash, email, email_verified, username)
values
    (:'userOneID', 'report-user-one', 'private-one@example.test', true, 'private-one'),
    (:'userTwoID', 'report-user-two', 'private-two@example.test', true, 'private-two'),
    (:'userThreeID', 'report-user-three', 'private-three@example.test', true, 'private-three');

insert into event (
    event_id,
    group_id,
    name,
    slug,
    description,
    timezone,
    event_category_id,
    event_kind_id,
    starts_at,
    ends_at,
    published
) values
    (
        :'eventID',
        :'groupID',
        'Target Sponsor Event',
        'target-sponsor-event',
        'Target report event',
        'UTC',
        :'eventCategoryID',
        'in-person',
        current_timestamp - interval '2 days',
        current_timestamp - interval '1 day',
        true
    ),
    (
        :'otherEventID',
        :'groupID',
        'Other Sponsor Event',
        'other-sponsor-event',
        'Other event whose metrics must not leak',
        'UTC',
        :'eventCategoryID',
        'in-person',
        current_timestamp - interval '4 days',
        current_timestamp - interval '3 days',
        true
    ),
    (
        :'zeroEventID',
        :'groupID',
        'Empty Sponsor Event',
        'empty-sponsor-event',
        'Event without report data',
        'UTC',
        :'eventCategoryID',
        'virtual',
        current_timestamp + interval '2 days',
        current_timestamp + interval '2 days 2 hours',
        true
    );

insert into group_sponsor (group_sponsor_id, group_id, name, logo_url)
values (:'sponsorID', :'groupID', 'Example Sponsor', 'https://example.com/sponsor.png');

insert into event_sponsor (event_id, group_sponsor_id, level)
values
    (:'eventID', :'sponsorID', 'Gold'),
    (:'otherEventID', :'sponsorID', 'Gold');

insert into event_views (event_id, day, total)
values
    (:'eventID', current_date, 100),
    (:'otherEventID', current_date, 999);

insert into event_attendee (event_id, user_id, status, checked_in)
values
    (:'eventID', :'userOneID', 'confirmed', true),
    (:'eventID', :'userTwoID', 'confirmed', false),
    (:'otherEventID', :'userThreeID', 'confirmed', true);

insert into event_sponsor_engagement_daily (
    event_id,
    group_sponsor_id,
    day,
    metric,
    visitor_hash
) values
    (:'eventID', :'sponsorID', current_date, 'impression', decode('01', 'hex')),
    (:'eventID', :'sponsorID', current_date, 'impression', decode('02', 'hex')),
    (:'eventID', :'sponsorID', current_date, 'impression', decode('03', 'hex')),
    (:'eventID', :'sponsorID', current_date, 'impression', decode('04', 'hex')),
    (:'eventID', :'sponsorID', current_date, 'click', decode('01', 'hex')),
    (:'otherEventID', :'sponsorID', current_date, 'impression', decode('05', 'hex')),
    (:'otherEventID', :'sponsorID', current_date, 'click', decode('05', 'hex'));

insert into event_sponsor_manual_engagement (
    event_id,
    group_sponsor_id,
    leads_count,
    conversations_count,
    meetings_count,
    notes,
    updated_by
) values
    (:'eventID', :'sponsorID', 3, 2, 1, 'Private organizer note', :'userOneID'),
    (:'otherEventID', :'sponsorID', 99, 99, 99, 'Other event note', :'userOneID');

insert into gtm_lead (
    gtm_lead_id,
    alliance_id,
    group_id,
    kind,
    stage,
    name,
    group_sponsor_id,
    payload
) values
    (
        :'targetLeadID',
        :'allianceID',
        :'groupID',
        'sponsor',
        'won',
        'Target event sponsorship',
        :'sponsorID',
        jsonb_build_object('event_id', :'eventID')
    ),
    (
        :'otherLeadID',
        :'allianceID',
        :'groupID',
        'sponsor',
        'won',
        'Other event sponsorship',
        :'sponsorID',
        jsonb_build_object('event_id', :'otherEventID')
    );

insert into gtm_sponsor_deliverable (gtm_lead_id, title, state)
values
    (:'targetLeadID', 'Target delivered', 'delivered'),
    (:'targetLeadID', 'Target pending', 'pending'),
    (:'otherLeadID', 'Other delivered', 'delivered');

select ensure_event_surveys(:'eventID');
insert into event_survey_response (event_id, audience, user_id, answers)
values
    (
        :'eventID',
        'sponsor-contact',
        :'userOneID',
        '{"answers":[
            {"question_id":"11111111-1111-4111-8111-111111111111","value":10},
            {"question_id":"22222222-2222-4222-8222-222222222222","value":5}
        ]}'::jsonb
    ),
    (
        :'eventID',
        'sponsor-contact',
        :'userTwoID',
        '{"answers":[
            {"question_id":"11111111-1111-4111-8111-111111111111","value":8},
            {"question_id":"22222222-2222-4222-8222-222222222222","value":4}
        ]}'::jsonb
    ),
    (
        :'eventID',
        'sponsor-contact',
        :'userThreeID',
        '{"answers":[
            {"question_id":"11111111-1111-4111-8111-111111111111","value":5},
            {"question_id":"22222222-2222-4222-8222-222222222222","value":3}
        ]}'::jsonb
    );

select lives_ok(
    format('select get_event_sponsor_report(%L::uuid)', :'eventID'),
    'aggregate sponsor report executes with event data'
);

select results_eq(
    format(
        $$
            select
                report->>'sponsor_count',
                report->>'total_impressions',
                report->>'total_clicks',
                report->>'click_through_rate',
                report->>'total_leads',
                report->>'total_conversations',
                report->>'total_meetings',
                report->>'promised_deliverables',
                report->>'delivered_deliverables',
                report->>'deliverable_completion_rate'
            from (select get_event_sponsor_report(%L::uuid) report) report
        $$,
        :'eventID'
    ),
    $$values ('1', '4', '1', '25.00', '3', '2', '1', '2', '1', '50.00')$$,
    'report totals and rates are event-scoped'
);

select results_eq(
    format(
        $$
            select
                report#>>'{event_outcomes,views}',
                report#>>'{event_outcomes,registrations}',
                report#>>'{event_outcomes,confirmed}',
                report#>>'{event_outcomes,check_ins}',
                report#>>'{event_outcomes,conversion_rate}'
            from (select get_event_sponsor_report(%L::uuid) report) report
        $$,
        :'eventID'
    ),
    $$values ('100', '2', '2', '1', '2.00')$$,
    'event outcomes match the privacy-safe growth funnel aggregates'
);

select results_eq(
    format(
        $$
            select
                sponsor->>'impressions',
                sponsor->>'clicks',
                sponsor->>'click_through_rate',
                sponsor->>'promised_deliverables',
                sponsor->>'delivered_deliverables',
                sponsor->>'deliverable_completion_rate'
            from jsonb_array_elements(
                get_event_sponsor_report(%L::uuid)->'sponsors'
            ) sponsor
        $$,
        :'eventID'
    ),
    $$values ('4', '1', '25.00', '2', '1', '50.00')$$,
    'per-sponsor performance excludes other-event engagement and deliverables'
);

select results_eq(
    format(
        $$
            select
                report->>'sponsor_count',
                report->>'click_through_rate',
                report->>'deliverable_completion_rate',
                report#>>'{event_outcomes,conversion_rate}'
            from (select get_event_sponsor_report(%L::uuid) report) report
        $$,
        :'zeroEventID'
    ),
    $$values ('0', '0', '0', '0')$$,
    'zero-data reports avoid divide-by-zero'
);

select is(
    get_event_sponsor_report(:'eventID')#>>'{sponsor_contact_survey,responses}',
    '3',
    'sponsor feedback appears only after the privacy threshold'
);

insert into event_sponsor_report_share (event_id, token, created_by)
values (:'eventID', :'shareToken', :'userOneID');

select ok(
    not exists (
        select 1
        from jsonb_array_elements(
            get_public_event_sponsor_report(:'shareToken')->'sponsors'
        ) sponsor
        where sponsor ? 'notes'
    ),
    'public reports remove organizer-only notes'
);

select is(
    get_public_event_sponsor_report(:'shareToken')->>'total_impressions',
    '4',
    'public reports retain aggregate performance'
);

select ok(
    get_public_event_sponsor_report(:'shareToken')::text not like '%private-%'
    and get_public_event_sponsor_report(:'shareToken')::text
        not like '%Private organizer note%',
    'public reports contain no attendee identity or private notes'
);

select * from finish();
rollback;
