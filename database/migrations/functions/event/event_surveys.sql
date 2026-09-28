-- Event-scoped post-event survey configuration, eligibility, responses and notifications.

create or replace function ensure_event_surveys(p_event_id uuid)
returns void as $$
declare
    v_audience text;
    v_prompt text;
begin
    foreach v_audience in array array['attendee', 'speaker', 'sponsor-contact']
    loop
        v_prompt := case v_audience
            when 'attendee' then 'How likely are you to recommend this event?'
            when 'speaker' then 'How likely are you to recommend speaking at this event?'
            else 'How likely are you to recommend sponsoring this event?'
        end;

        insert into event_survey (event_id, audience, questions)
        select p_event_id, v_audience, jsonb_build_array(
            jsonb_build_object(
                'id', '11111111-1111-4111-8111-111111111111',
                'kind', 'nps',
                'prompt', v_prompt,
                'required', true,
                'options', '[]'::jsonb,
                'min', 0,
                'max', 10
            ),
            jsonb_build_object(
                'id', '22222222-2222-4222-8222-222222222222',
                'kind', 'numeric-scale',
                'prompt', 'How would you rate your overall experience?',
                'required', true,
                'options', '[]'::jsonb,
                'min', 1,
                'max', 5
            ),
            jsonb_build_object(
                'id', '33333333-3333-4333-8333-333333333333',
                'kind', 'free-text',
                'prompt', 'What worked well, and what could be improved?',
                'required', false,
                'options', '[]'::jsonb
            )
        )
        from event
        where event_id = p_event_id
        on conflict (event_id, audience) do nothing;
    end loop;
end;
$$ language plpgsql;

create or replace function event_survey_user_is_eligible(
    p_event_id uuid,
    p_audience text,
    p_user_id uuid
) returns boolean as $$
begin
    if p_audience = 'attendee' then
        return exists (
            select 1 from event_attendee
            where event_id = p_event_id
              and user_id = p_user_id
              and (status = 'confirmed' or checked_in)
        );
    elsif p_audience = 'speaker' then
        return exists (
            select 1 from event_speaker
            where event_id = p_event_id and user_id = p_user_id
            union all
            select 1
            from session s
            join session_speaker ss using (session_id)
            where s.event_id = p_event_id and ss.user_id = p_user_id
        );
    elsif p_audience = 'sponsor-contact' then
        return exists (
            select 1
            from event e
            join event_sponsor es using (event_id)
            join gtm_lead lead
              on lead.group_id = e.group_id
             and lead.group_sponsor_id = es.group_sponsor_id
             and lead.kind = 'sponsor'
             and lead.stage in ('won', 'delivered', 'renewal')
            join gtm_sponsor_contact contact using (gtm_lead_id)
            where e.event_id = p_event_id
              and contact.user_id = p_user_id
        );
    end if;
    return false;
end;
$$ language plpgsql stable;

create or replace function get_event_survey_for_user(
    p_alliance_id uuid,
    p_event_id uuid,
    p_audience text,
    p_user_id uuid
) returns jsonb as $$
declare
    v_result jsonb;
begin
    if p_audience not in ('attendee', 'speaker', 'sponsor-contact') then
        raise exception 'invalid survey audience';
    end if;
    perform ensure_event_surveys(p_event_id);

    select jsonb_build_object(
        'event', get_event_summary(p_alliance_id, e.group_id, e.event_id),
        'audience', survey.audience,
        'questions', survey.questions,
        'submitted', response.user_id is not null
    )
    into v_result
    from event e
    join "group" g using (group_id)
    join event_survey survey using (event_id)
    left join event_survey_response response
      on response.event_id = survey.event_id
     and response.audience = survey.audience
     and response.user_id = p_user_id
    where e.event_id = p_event_id
      and g.alliance_id = p_alliance_id
      and survey.audience = p_audience
      and survey.enabled
      and e.published and not e.deleted and not e.canceled
      and coalesce(e.ends_at, e.starts_at) < current_timestamp
      and event_survey_user_is_eligible(e.event_id, survey.audience, p_user_id);

    return v_result;
end;
$$ language plpgsql;

create or replace function submit_event_survey_response(
    p_alliance_id uuid,
    p_event_id uuid,
    p_audience text,
    p_user_id uuid,
    p_answers jsonb
) returns void as $$
declare
    v_questions jsonb;
begin
    perform ensure_event_surveys(p_event_id);

    select survey.questions into v_questions
    from event_survey survey
    join event e using (event_id)
    join "group" g using (group_id)
    where survey.event_id = p_event_id
      and survey.audience = p_audience
      and g.alliance_id = p_alliance_id
      and survey.enabled
      and e.published and not e.deleted and not e.canceled
      and coalesce(e.ends_at, e.starts_at) < current_timestamp
      and event_survey_user_is_eligible(e.event_id, survey.audience, p_user_id);

    if v_questions is null then
        raise exception 'survey unavailable or user is not eligible';
    end if;

    perform validate_questionnaire_answers_payload(v_questions, p_answers);

    if exists (
        select 1
        from jsonb_array_elements(p_answers->'answers') answer
        where jsonb_typeof(answer->'value') = 'string'
          and length(answer->>'value') > 4000
    ) then
        raise exception 'survey free-text answer is too long';
    end if;

    insert into event_survey_response (event_id, audience, user_id, answers)
    values (p_event_id, p_audience, p_user_id, p_answers);
exception when unique_violation then
    raise exception 'survey response already submitted';
end;
$$ language plpgsql;

create or replace function get_event_survey_dashboard(
    p_group_id uuid,
    p_event_id uuid,
    p_audience text default null
) returns jsonb as $$
declare
    v_result jsonb;
begin
    if p_audience is not null
       and p_audience not in ('attendee', 'speaker', 'sponsor-contact') then
        raise exception 'invalid survey audience';
    end if;
    perform ensure_event_surveys(p_event_id);

    with audiences as (
        select unnest(array['attendee', 'speaker', 'sponsor-contact']) audience
    ),
    eligible as (
        select audience, count(*)::bigint total
        from audiences
        cross join lateral (
            select u.user_id from "user" u
            where event_survey_user_is_eligible(p_event_id, audience, u.user_id)
        ) users
        group by audience
    ),
    scored as (
        select
            r.audience,
            r.user_id,
            r.submitted_at,
            r.answers,
            (nps.answer->>'value')::int nps,
            (rating.answer->>'value')::int rating,
            nullif(btrim(regexp_replace(comment.answer->>'value', '[[:cntrl:]]', ' ', 'g')), '') comment
        from event_survey_response r
        left join lateral (
            select answer from jsonb_array_elements(r.answers->'answers') answer
            where answer->>'question_id' = '11111111-1111-4111-8111-111111111111'
        ) nps on true
        left join lateral (
            select answer from jsonb_array_elements(r.answers->'answers') answer
            where answer->>'question_id' = '22222222-2222-4222-8222-222222222222'
        ) rating on true
        left join lateral (
            select answer from jsonb_array_elements(r.answers->'answers') answer
            where answer->>'question_id' = '33333333-3333-4333-8333-333333333333'
        ) comment on true
        where r.event_id = p_event_id
          and (p_audience is null or r.audience = p_audience)
    ),
    metrics as (
        select jsonb_agg(jsonb_build_object(
            'audience', audiences.audience,
            'eligible', coalesce(eligible.total, 0),
            'responses', count(scored.user_id),
            'response_rate', case when coalesce(eligible.total, 0) = 0 then 0
                else round(count(scored.user_id)::numeric * 100 / eligible.total, 1) end,
            'promoters', count(*) filter (where scored.nps >= 9),
            'passives', count(*) filter (where scored.nps between 7 and 8),
            'detractors', count(*) filter (where scored.nps <= 6),
            'nps_score', case when count(scored.nps) = 0 then null else round(
                100.0 * (
                    count(*) filter (where scored.nps >= 9)
                    - count(*) filter (where scored.nps <= 6)
                ) / count(scored.nps), 1) end,
            'average_rating', round(avg(scored.rating), 2)
        ) order by audiences.audience) data
        from audiences
        left join eligible using (audience)
        left join scored using (audience)
        where p_audience is null or audiences.audience = p_audience
        group by ()
    )
    select jsonb_build_object(
        'metrics', coalesce(metrics.data, '[]'::jsonb),
        'responses', coalesce((
            select jsonb_agg(jsonb_build_object(
                'audience', audience,
                'submitted_at', extract(epoch from submitted_at)::bigint,
                'nps', nps,
                'rating', rating,
                -- Deliberately anonymous: never return user_id/name/email with free text.
                'comment', comment
            ) order by submitted_at desc)
            from scored
        ), '[]'::jsonb)
    ) into v_result
    from metrics
    where exists (
        select 1 from event where event_id = p_event_id and group_id = p_group_id and not deleted
    );

    if v_result is null then
        raise exception 'event not found';
    end if;
    return v_result;
end;
$$ language plpgsql;

create or replace function enqueue_due_event_survey_notifications(p_base_url text)
returns int as $$
declare
    v_count int := 0;
    v_item record;
    v_kind text;
begin
    if not pg_try_advisory_xact_lock(hashtextextended('ocg:event-survey-enqueue', 0)) then
        return 0;
    end if;

    for v_item in
        with ended_events as (
            select e.*, g.alliance_id, a.name alliance_name, g.name group_name,
                   a.display_name alliance_display_name
            from event e
            join "group" g using (group_id)
            join alliance a using (alliance_id)
            where e.published and not e.deleted and not e.canceled and not e.test_event
              and coalesce(e.ends_at, e.starts_at) between current_timestamp - interval '30 days'
                                                        and current_timestamp
        ),
        candidates as (
            select e.*, audiences.audience, u.user_id,
                   case when request_state.user_id is null then 'request' else 'reminder' end kind
            from ended_events e
            cross join unnest(array['attendee', 'speaker', 'sponsor-contact'])
                as audiences(audience)
            join "user" u on u.email_verified
              and event_survey_user_is_eligible(e.event_id, audiences.audience, u.user_id)
            left join event_survey_notification request_state
              on request_state.event_id = e.event_id
             and request_state.audience = audiences.audience
             and request_state.user_id = u.user_id and request_state.kind = 'request'
            left join event_survey_notification reminder_state
              on reminder_state.event_id = e.event_id
             and reminder_state.audience = audiences.audience
             and reminder_state.user_id = u.user_id and reminder_state.kind = 'reminder'
            left join event_survey_response response
              on response.event_id = e.event_id
             and response.audience = audiences.audience
             and response.user_id = u.user_id
            where response.user_id is null
              and (
                (request_state.user_id is null
                 and coalesce(e.ends_at, e.starts_at) <= current_timestamp)
                or
                (request_state.enqueued_at <= current_timestamp - interval '3 days'
                 and reminder_state.user_id is null)
              )
        )
        select * from candidates order by event_id, audience, user_id
    loop
        perform ensure_event_surveys(v_item.event_id);
        v_kind := v_item.kind;
        insert into event_survey_notification (event_id, audience, user_id, kind)
        values (v_item.event_id, v_item.audience, v_item.user_id, v_kind)
        on conflict do nothing;
        if found then
            perform enqueue_notification(
                case v_kind when 'request' then 'event-survey-request'
                            else 'event-survey-reminder' end,
                jsonb_build_object(
                    'event_name', v_item.name,
                    'group_name', v_item.group_name,
                    'audience', v_item.audience,
                    'reminder', v_kind = 'reminder',
                    'link', format(
                        '%s/%s/event/%s/survey/%s',
                        regexp_replace(coalesce(p_base_url, ''), '/+$', ''),
                        v_item.alliance_name,
                        v_item.event_id,
                        v_item.audience
                    ),
                    'theme', coalesce((select theme from site order by created_at desc limit 1), '{}'::jsonb)
                ),
                '[]'::jsonb,
                array[v_item.user_id]
            );
            v_count := v_count + 1;
        end if;
    end loop;
    return v_count;
end;
$$ language plpgsql;
