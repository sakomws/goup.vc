create or replace function gtm_lead_json(p_lead gtm_lead)
returns jsonb language sql stable as $$
    select jsonb_strip_nulls(jsonb_build_object(
        'gtm_lead_id', p_lead.gtm_lead_id,
        'alliance_id', p_lead.alliance_id,
        'group_id', p_lead.group_id,
        'kind', p_lead.kind,
        'stage', p_lead.stage,
        'name', p_lead.name,
        'org_name', p_lead.org_name,
        'email', p_lead.email,
        'website_url', p_lead.website_url,
        'linkedin_url', p_lead.linkedin_url,
        'landscape_entry_id', p_lead.landscape_entry_id,
        'user_id', p_lead.user_id,
        'group_sponsor_id', p_lead.group_sponsor_id,
        'owner_user_id', p_lead.owner_user_id,
        'score', p_lead.score,
        'estimated_value_cents', p_lead.estimated_value_cents,
        'currency', p_lead.currency,
        'next_action_at', extract(epoch from p_lead.next_action_at)::bigint,
        'renewal_at', extract(epoch from p_lead.renewal_at)::bigint,
        'lost_reason', p_lead.lost_reason,
        'source', p_lead.source,
        'notes', p_lead.notes,
        'payload', p_lead.payload,
        'created_at', extract(epoch from p_lead.created_at)::bigint,
        'updated_at', extract(epoch from p_lead.updated_at)::bigint
    ));
$$;
