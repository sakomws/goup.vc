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
    if not exists (
        select 1 from event
        where event_id = p_event_id and not deleted
    ) then
        raise exception 'event not found';
    end if;

    with target_event as (
        select event_id, group_id, name, starts_at
        from event
        where event_id = p_event_id and not deleted
    ),
    registration_users as (
        select user_id from event_registration_attribution where event_id = p_event_id
        union
        select user_id from event_attendee where event_id = p_event_id
        union
        select user_id from event_waitlist where event_id = p_event_id
        union
        select user_id from event_invitation_request where event_id = p_event_id
        union
        select user_id from event_purchase where event_id = p_event_id
    ),
    event_outcomes as (
        select
            coalesce((select sum(total) from event_views where event_id = p_event_id), 0)::bigint views,
            (select count(*) from registration_users)::bigint registrations,
            count(*) filter (where attendee.status = 'confirmed')::bigint confirmed,
            count(*) filter (
                where attendee.status = 'confirmed' and attendee.checked_in
            )::bigint check_ins
        from event_attendee attendee
        where attendee.event_id = p_event_id
    ),
    sponsor_rows as (
        select
            gs.group_sponsor_id,
            gs.name,
            es.level,
            coalesce(metrics.impressions, 0)::bigint impressions,
            coalesce(metrics.clicks, 0)::bigint clicks,
            case when coalesce(metrics.impressions, 0) = 0 then 0
                else round(
                    coalesce(metrics.clicks, 0)::numeric * 100 / metrics.impressions,
                    2
                )
            end click_through_rate,
            coalesce(manual.leads_count, 0) leads_count,
            coalesce(manual.conversations_count, 0) conversations_count,
            coalesce(manual.meetings_count, 0) meetings_count,
            manual.notes,
            coalesce(deliverables.promised, 0)::bigint promised_deliverables,
            coalesce(deliverables.delivered, 0)::bigint delivered_deliverables,
            case when coalesce(deliverables.promised, 0) = 0 then 0
                else round(
                    coalesce(deliverables.delivered, 0)::numeric
                    * 100 / deliverables.promised,
                    2
                )
            end deliverable_completion_rate
        from target_event e
        join event_sponsor es using (event_id)
        join group_sponsor gs using (group_sponsor_id)
        left join lateral (
            select
                count(*) filter (where metric = 'impression') impressions,
                count(*) filter (where metric = 'click') clicks
            from event_sponsor_engagement_daily metric
            where metric.event_id = es.event_id
              and metric.group_sponsor_id = es.group_sponsor_id
        ) metrics on true
        left join event_sponsor_manual_engagement manual
          on manual.event_id = es.event_id
         and manual.group_sponsor_id = es.group_sponsor_id
        left join lateral (
            select
                count(*) promised,
                count(*) filter (where deliverable.state = 'delivered') delivered
            from gtm_lead lead
            join gtm_sponsor_deliverable deliverable using (gtm_lead_id)
            where lead.group_sponsor_id = es.group_sponsor_id
              and lead.group_id = e.group_id
              and lead.payload->>'event_id' = es.event_id::text
        ) deliverables on true
    ),
    sponsor_totals as (
        select
            count(*)::bigint sponsor_count,
            coalesce(sum(impressions), 0)::bigint total_impressions,
            coalesce(sum(clicks), 0)::bigint total_clicks,
            coalesce(sum(leads_count), 0)::bigint total_leads,
            coalesce(sum(conversations_count), 0)::bigint total_conversations,
            coalesce(sum(meetings_count), 0)::bigint total_meetings,
            coalesce(sum(promised_deliverables), 0)::bigint promised_deliverables,
            coalesce(sum(delivered_deliverables), 0)::bigint delivered_deliverables
        from sponsor_rows
    )
    select jsonb_build_object(
        'event_id', e.event_id,
        'event_name', e.name,
        'starts_at', extract(epoch from e.starts_at)::bigint,
        'sponsor_count', totals.sponsor_count,
        'total_impressions', totals.total_impressions,
        'total_clicks', totals.total_clicks,
        'click_through_rate', case when totals.total_impressions = 0 then 0
            else round(totals.total_clicks::numeric * 100 / totals.total_impressions, 2)
        end,
        'total_leads', totals.total_leads,
        'total_conversations', totals.total_conversations,
        'total_meetings', totals.total_meetings,
        'promised_deliverables', totals.promised_deliverables,
        'delivered_deliverables', totals.delivered_deliverables,
        'deliverable_completion_rate', case when totals.promised_deliverables = 0 then 0
            else round(
                totals.delivered_deliverables::numeric
                * 100 / totals.promised_deliverables,
                2
            )
        end,
        'event_outcomes', jsonb_build_object(
            'views', outcomes.views,
            'registrations', outcomes.registrations,
            'confirmed', outcomes.confirmed,
            'check_ins', outcomes.check_ins,
            'conversion_rate', case when outcomes.views = 0 then 0
                else round(outcomes.confirmed::numeric * 100 / outcomes.views, 2)
            end
        ),
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
                'group_sponsor_id', sponsor.group_sponsor_id,
                'name', sponsor.name,
                'level', sponsor.level,
                'impressions', sponsor.impressions,
                'clicks', sponsor.clicks,
                'click_through_rate', sponsor.click_through_rate,
                'leads_count', sponsor.leads_count,
                'conversations_count', sponsor.conversations_count,
                'meetings_count', sponsor.meetings_count,
                'notes', sponsor.notes,
                'promised_deliverables', sponsor.promised_deliverables,
                'delivered_deliverables', sponsor.delivered_deliverables,
                'deliverable_completion_rate', sponsor.deliverable_completion_rate
            ) order by sponsor.name)
            from sponsor_rows sponsor
        ), '[]'::jsonb)
    ) into v_result
    from target_event e
    cross join sponsor_totals totals
    cross join event_outcomes outcomes;

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
