create or replace function gtm_agent_draft_json(p_draft gtm_agent_draft)
returns jsonb language sql stable as $$
    select jsonb_strip_nulls(jsonb_build_object(
        'gtm_agent_draft_id', p_draft.gtm_agent_draft_id,
        'alliance_id', p_draft.alliance_id,
        'group_id', p_draft.group_id,
        'gtm_lead_id', p_draft.gtm_lead_id,
        'agent_id', p_draft.agent_id,
        'status', p_draft.status,
        'title', p_draft.title,
        'body', p_draft.body,
        'suggested_stage', p_draft.suggested_stage,
        'payload', p_draft.payload,
        'created_at', extract(epoch from p_draft.created_at)::bigint,
        'reviewed_at', extract(epoch from p_draft.reviewed_at)::bigint,
        'reviewed_by', p_draft.reviewed_by
    ));
$$;
