create or replace function list_gtm_agent_drafts(
    p_alliance_id uuid,
    p_filters jsonb
) returns jsonb language plpgsql stable as $$
declare
    v_lead_id uuid := nullif(p_filters->>'gtm_lead_id', '')::uuid;
    v_status text := nullif(trim(p_filters->>'status'), '');
    v_group_id uuid := nullif(p_filters->>'group_id', '')::uuid;
    v_drafts jsonb;
begin
    select coalesce(
        jsonb_agg(gtm_agent_draft_json(d) order by d.created_at desc),
        '[]'::jsonb
    )
    into v_drafts
    from gtm_agent_draft d
    where d.alliance_id = p_alliance_id
      and (v_lead_id is null or d.gtm_lead_id = v_lead_id)
      and (v_status is null or d.status = v_status)
      and (v_group_id is null or d.group_id = v_group_id);

    return jsonb_build_object('drafts', coalesce(v_drafts, '[]'::jsonb));
end;
$$;
