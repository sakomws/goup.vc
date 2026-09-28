begin;
select plan(24);

\set allianceID '5e150000-0000-0000-0000-000000000001'
\set groupCategoryID '5e150000-0000-0000-0000-000000000002'
\set eventCategoryID '5e150000-0000-0000-0000-000000000003'
\set groupID '5e150000-0000-0000-0000-000000000004'
\set otherGroupID '5e150000-0000-0000-0000-000000000005'
\set eventID '5e150000-0000-0000-0000-000000000006'
\set attendeeOneID '5e150000-0000-0000-0000-000000000007'
\set attendeeTwoID '5e150000-0000-0000-0000-000000000008'
\set speakerID '5e150000-0000-0000-0000-000000000009'

select has_table('event_survey');
select has_table('event_survey_response');
select has_table('event_survey_notification');
select has_pk('event_survey');
select has_pk('event_survey_response');
select has_pk('event_survey_notification');
select has_function('ensure_event_surveys', array['uuid']::name[]);
select has_function(
    'event_survey_user_is_eligible',
    array['uuid', 'text', 'uuid']::name[]
);
select has_function(
    'get_event_survey_for_user',
    array['uuid', 'uuid', 'text', 'uuid']::name[]
);
select has_function(
    'submit_event_survey_response',
    array['uuid', 'uuid', 'text', 'uuid', 'jsonb']::name[]
);
select has_function(
    'get_event_survey_dashboard',
    array['uuid', 'uuid', 'text']::name[]
);
select has_function('enqueue_due_event_survey_notifications', array['text']::name[]);

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
    'survey-alliance',
    'Survey Alliance',
    'Alliance for event survey tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Survey groups');

insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Survey events');

insert into "group" (
    group_id,
    alliance_id,
    group_category_id,
    name,
    slug
) values
    (:'groupID', :'allianceID', :'groupCategoryID', 'Survey Group', 'survey-group'),
    (:'otherGroupID', :'allianceID', :'groupCategoryID', 'Other Group', 'other-survey-group');

insert into "user" (user_id, auth_hash, email, email_verified, username)
values
    (
        :'attendeeOneID',
        'survey-attendee-one-hash',
        'private-attendee-one@example.test',
        true,
        'private-attendee-one'
    ),
    (
        :'attendeeTwoID',
        'survey-attendee-two-hash',
        'private-attendee-two@example.test',
        true,
        'private-attendee-two'
    ),
    (
        :'speakerID',
        'survey-speaker-hash',
        'private-speaker@example.test',
        true,
        'private-speaker'
    );

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
    published,
    published_at
) values (
    :'eventID',
    :'groupID',
    'Survey Event',
    'survey-event',
    'An event with completed survey responses',
    'UTC',
    :'eventCategoryID',
    'in-person',
    current_timestamp - interval '2 days',
    current_timestamp - interval '1 day',
    true,
    current_timestamp - interval '3 days'
);

insert into event_attendee (event_id, user_id, status)
values
    (:'eventID', :'attendeeOneID', 'confirmed'),
    (:'eventID', :'attendeeTwoID', 'confirmed');

insert into event_speaker (event_id, user_id)
values (:'eventID', :'speakerID');

select throws_ok(
    format(
        'select get_event_survey_dashboard(%L::uuid, %L::uuid, null)',
        :'otherGroupID',
        :'eventID'
    ),
    'P0001',
    'event not found',
    'dashboard rejects an event belonging to another group'
);

select is(
    (select count(*) from event_survey where event_id = :'eventID'),
    0::bigint,
    'cross-group dashboard access does not initialize surveys'
);

select lives_ok(
    format(
        $$
            select submit_event_survey_response(
                %L::uuid,
                %L::uuid,
                'attendee',
                %L::uuid,
                '{"answers":[
                    {"question_id":"11111111-1111-4111-8111-111111111111","value":10},
                    {"question_id":"22222222-2222-4222-8222-222222222222","value":5},
                    {"question_id":"33333333-3333-4333-8333-333333333333","value":"Wonderful hosts"}
                ]}'::jsonb
            )
        $$,
        :'allianceID',
        :'eventID',
        :'attendeeOneID'
    ),
    'first attendee survey response is accepted'
);

select lives_ok(
    format(
        $$
            select submit_event_survey_response(
                %L::uuid,
                %L::uuid,
                'attendee',
                %L::uuid,
                '{"answers":[
                    {"question_id":"11111111-1111-4111-8111-111111111111","value":6},
                    {"question_id":"22222222-2222-4222-8222-222222222222","value":3},
                    {"question_id":"33333333-3333-4333-8333-333333333333","value":"More discussion time"}
                ]}'::jsonb
            )
        $$,
        :'allianceID',
        :'eventID',
        :'attendeeTwoID'
    ),
    'second attendee survey response is accepted'
);

select lives_ok(
    format(
        $$
            select submit_event_survey_response(
                %L::uuid,
                %L::uuid,
                'speaker',
                %L::uuid,
                '{"answers":[
                    {"question_id":"11111111-1111-4111-8111-111111111111","value":8},
                    {"question_id":"22222222-2222-4222-8222-222222222222","value":4}
                ]}'::jsonb
            )
        $$,
        :'allianceID',
        :'eventID',
        :'speakerID'
    ),
    'speaker survey response is accepted'
);

select lives_ok(
    format(
        'select get_event_survey_dashboard(%L::uuid, %L::uuid, null)',
        :'groupID',
        :'eventID'
    ),
    'dashboard aggregates real responses without nested aggregate errors'
);

select results_eq(
    format(
        $$
            select
                metric->>'audience',
                metric->>'eligible',
                metric->>'responses',
                metric->>'response_rate',
                metric->>'promoters',
                metric->>'passives',
                metric->>'detractors',
                metric->>'nps_score',
                metric->>'average_rating'
            from jsonb_array_elements(
                get_event_survey_dashboard(%L::uuid, %L::uuid, null)->'metrics'
            ) metric
            where metric->>'audience' = 'attendee'
        $$,
        :'groupID',
        :'eventID'
    ),
    $$values ('attendee', '2', '2', '100.0', '1', '0', '1', '0.0', '4.00')$$,
    'attendee metrics preserve eligibility, response rate, NPS segments, and rating'
);

select results_eq(
    format(
        $$
            select
                metric->>'eligible',
                metric->>'responses',
                metric->>'promoters',
                metric->>'passives',
                metric->>'detractors',
                metric->>'nps_score',
                metric->>'average_rating'
            from jsonb_array_elements(
                get_event_survey_dashboard(%L::uuid, %L::uuid, null)->'metrics'
            ) metric
            where metric->>'audience' = 'speaker'
        $$,
        :'groupID',
        :'eventID'
    ),
    $$values ('1', '1', '0', '1', '0', '0.0', '4.00')$$,
    'speaker metrics preserve audience-specific counts'
);

select results_eq(
    format(
        $$
            with dashboard as (
                select get_event_survey_dashboard(
                    %L::uuid,
                    %L::uuid,
                    'attendee'
                ) data
            )
            select
                jsonb_array_length(data->'metrics'),
                jsonb_array_length(data->'responses'),
                (
                    select bool_and(response->>'audience' = 'attendee')
                    from jsonb_array_elements(data->'responses') response
                )
            from dashboard
        $$,
        :'groupID',
        :'eventID'
    ),
    $$values (1, 2, true)$$,
    'audience filters apply to metrics and response rows'
);

select ok(
    not exists (
        select 1
        from jsonb_array_elements(
            get_event_survey_dashboard(:'groupID', :'eventID', null)->'responses'
        ) response
        cross join lateral jsonb_object_keys(response) key
        where key in ('user_id', 'name', 'username', 'email')
    ),
    'anonymous response rows exclude identity fields'
);

select ok(
    get_event_survey_dashboard(:'groupID', :'eventID', null)::text
        not like '%private-%',
    'dashboard payload does not leak usernames or email addresses'
);

select lives_ok(
    $$
        select validate_questionnaire_answers_payload(
            '[
                {
                    "id": "11111111-1111-4111-8111-111111111111",
                    "kind": "nps",
                    "prompt": "Recommend?",
                    "required": true,
                    "options": [],
                    "min": 0,
                    "max": 10
                },
                {
                    "id": "22222222-2222-4222-8222-222222222222",
                    "kind": "numeric-scale",
                    "prompt": "Rate?",
                    "required": true,
                    "options": [],
                    "min": 1,
                    "max": 5
                }
            ]'::jsonb,
            '{
                "answers": [
                    {
                        "question_id": "11111111-1111-4111-8111-111111111111",
                        "value": 10
                    },
                    {
                        "question_id": "22222222-2222-4222-8222-222222222222",
                        "value": 5
                    }
                ]
            }'::jsonb
        )
    $$,
    'numeric scale and NPS answers are supported'
);

select * from finish();
rollback;
