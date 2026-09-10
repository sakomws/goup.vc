create or replace function get_gtm_lead(
    p_alliance_id uuid,
    p_gtm_lead_id uuid
) returns jsonb language plpgsql stable as $$
declare
    v_lead gtm_lead;
    v_activities jsonb;
    v_drafts jsonb;
begin
    select * into v_lead
    from gtm_lead
    where gtm_lead_id = p_gtm_lead_id
      and alliance_id = p_alliance_id;

    if not found then
        return null;
    end if;

    select coalesce(jsonb_agg(jsonb_strip_nulls(jsonb_build_object(
        'gtm_lead_activity_id', a.gtm_lead_activity_id,
        'gtm_lead_id', a.gtm_lead_id,
        'actor_user_id', a.actor_user_id,
        'agent_id', a.agent_id,
        'kind', a.kind,
        'body', a.body,
        'details', a.details,
        'created_at', extract(epoch from a.created_at)::bigint
    )) order by a.created_at desc), '[]'::jsonb)
    into v_activities
    from gtm_lead_activity a
    where a.gtm_lead_id = p_gtm_lead_id;

    select coalesce(
        jsonb_agg(gtm_agent_draft_json(d) order by d.created_at desc),
        '[]'::jsonb
    )
    into v_drafts
    from gtm_agent_draft d
    where d.gtm_lead_id = p_gtm_lead_id;

    return gtm_lead_json(v_lead)
        || jsonb_build_object(
            'activities', coalesce(v_activities, '[]'::jsonb),
            'drafts', coalesce(v_drafts, '[]'::jsonb)
        );
end;
$$;
