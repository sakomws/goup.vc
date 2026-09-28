-- Sponsor engagement uses a browser-generated, per-tab nonce. Only its one-way
-- daily hash is retained; no user, IP address, user-agent, or attendee data is stored.
create or replace function record_event_sponsor_engagement(
    p_event_id uuid,
    p_group_sponsor_id uuid,
    p_metric text,
    p_session_nonce uuid
) returns boolean as $$
declare
    v_inserted boolean;
begin
    if p_metric not in ('impression', 'click') then
        raise exception 'invalid sponsor engagement metric';
    end if;

    insert into event_sponsor_engagement_daily (
        event_id, group_sponsor_id, metric, visitor_hash
    )
    select
        p_event_id,
        p_group_sponsor_id,
        p_metric,
        digest(p_session_nonce::text || ':' || current_date::text, 'sha256')
    from event_sponsor es
    join event e using (event_id)
    where es.event_id = p_event_id
      and es.group_sponsor_id = p_group_sponsor_id
      and e.published and not e.deleted
    on conflict do nothing
    returning true into v_inserted;

    return coalesce(v_inserted, false);
end;
$$ language plpgsql;

create or replace function get_event_sponsor_report(
    p_event_id uuid
) returns jsonb as $$
declare
    v_result jsonb;
begin
    select jsonb_build_object(
        'event_id', e.event_id,
        'event_name', e.name,
        'starts_at', extract(epoch from e.starts_at)::bigint,
        'sponsor_contact_survey', (
            select case when count(*) < 3 then null else jsonb_build_object(
                'responses', count(*),
                'promoters', count(*) filter (where nps >= 9),
                'passives', count(*) filter (where nps between 7 and 8),
                'detractors', count(*) filter (where nps <= 6),
                'nps_score', round(100.0 * (
                    count(*) filter (where nps >= 9)
                    - count(*) filter (where nps <= 6)
                ) / count(*), 1),
                'average_rating', round(avg(rating), 2)
            ) end
            from (
                select
                    (select (answer->>'value')::int
                     from jsonb_array_elements(response.answers->'answers') answer
                     where answer->>'question_id' = '11111111-1111-4111-8111-111111111111') nps,
                    (select (answer->>'value')::int
                     from jsonb_array_elements(response.answers->'answers') answer
                     where answer->>'question_id' = '22222222-2222-4222-8222-222222222222') rating
                from event_survey_response response
                where response.event_id = e.event_id
                  and response.audience = 'sponsor-contact'
            ) survey_response
            where nps is not null
        ),
        'sponsors', coalesce((
            select jsonb_agg(jsonb_build_object(
                'group_sponsor_id', gs.group_sponsor_id,
                'name', gs.name,
                'level', es.level,
                'impressions', coalesce(metrics.impressions, 0),
                'clicks', coalesce(metrics.clicks, 0),
                'leads_count', coalesce(manual.leads_count, 0),
                'conversations_count', coalesce(manual.conversations_count, 0),
                'meetings_count', coalesce(manual.meetings_count, 0),
                'notes', manual.notes,
                'promised_deliverables', coalesce(deliverables.promised, 0),
                'delivered_deliverables', coalesce(deliverables.delivered, 0)
            ) order by gs.name)
            from event_sponsor es
            join group_sponsor gs using (group_sponsor_id)
            left join lateral (
                select
                    count(*) filter (where metric = 'impression') as impressions,
                    count(*) filter (where metric = 'click') as clicks
                from event_sponsor_engagement_daily metric
                where metric.event_id = es.event_id
                  and metric.group_sponsor_id = es.group_sponsor_id
            ) metrics on true
            left join event_sponsor_manual_engagement manual
              on manual.event_id = es.event_id
             and manual.group_sponsor_id = es.group_sponsor_id
            left join lateral (
                select
                    count(*) as promised,
                    count(*) filter (where d.state = 'delivered') as delivered
                from gtm_lead lead
                join gtm_sponsor_deliverable d using (gtm_lead_id)
                where lead.group_sponsor_id = es.group_sponsor_id
                  and lead.group_id = e.group_id
            ) deliverables on true
            where es.event_id = e.event_id
        ), '[]'::jsonb)
    ) into v_result
    from event e
    where e.event_id = p_event_id and not e.deleted;

    if v_result is null then
        raise exception 'event not found';
    end if;
    return v_result;
end;
$$ language plpgsql stable;

create or replace function update_event_sponsor_manual_engagement(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_group_sponsor_id uuid,
    p_input jsonb
) returns void as $$
begin
    insert into event_sponsor_manual_engagement (
        event_id, group_sponsor_id, leads_count, conversations_count,
        meetings_count, notes, updated_by
    )
    select
        p_event_id,
        p_group_sponsor_id,
        coalesce((p_input->>'leads_count')::integer, 0),
        coalesce((p_input->>'conversations_count')::integer, 0),
        coalesce((p_input->>'meetings_count')::integer, 0),
        nullif(btrim(p_input->>'notes'), ''),
        p_actor_user_id
    from event e
    join event_sponsor es using (event_id)
    where e.event_id = p_event_id
      and e.group_id = p_group_id
      and es.group_sponsor_id = p_group_sponsor_id
    on conflict (event_id, group_sponsor_id) do update set
        leads_count = excluded.leads_count,
        conversations_count = excluded.conversations_count,
        meetings_count = excluded.meetings_count,
        notes = excluded.notes,
        updated_by = excluded.updated_by,
        updated_at = current_timestamp;

    if not found then
        raise exception 'event sponsor not found';
    end if;
end;
$$ language plpgsql;

create or replace function create_event_sponsor_report_share(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid
) returns uuid as $$
declare
    v_token uuid;
begin
    insert into event_sponsor_report_share (event_id, token, created_by, revoked_at)
    select p_event_id, gen_random_uuid(), p_actor_user_id, null
    from event
    where event_id = p_event_id and group_id = p_group_id and not deleted
    on conflict (event_id) do update set
        token = gen_random_uuid(),
        created_by = excluded.created_by,
        created_at = current_timestamp,
        revoked_at = null
    returning token into v_token;

    if v_token is null then
        raise exception 'event not found';
    end if;
    return v_token;
end;
$$ language plpgsql;

create or replace function revoke_event_sponsor_report_share(
    p_group_id uuid,
    p_event_id uuid
) returns void as $$
begin
    update event_sponsor_report_share share
    set revoked_at = current_timestamp
    from event e
    where share.event_id = p_event_id
      and e.event_id = share.event_id
      and e.group_id = p_group_id;
end;
$$ language plpgsql;

create or replace function get_public_event_sponsor_report(
    p_token uuid
) returns jsonb as $$
    with active_report as (
        select get_event_sponsor_report(share.event_id) as report
        from event_sponsor_report_share share
        join event e using (event_id)
        where share.token = p_token
          and share.revoked_at is null
          and not e.deleted
    )
    select jsonb_set(
        report,
        '{sponsors}',
        coalesce((
            select jsonb_agg(sponsor - 'notes')
            from jsonb_array_elements(report->'sponsors') sponsor
        ), '[]'::jsonb)
    )
    from active_report;
$$ language sql stable;
