begin;
select plan(13);

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
