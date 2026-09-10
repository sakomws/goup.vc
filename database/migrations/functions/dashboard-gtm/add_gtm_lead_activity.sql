create or replace function add_gtm_lead_activity(
    p_actor_user_id uuid,
    p_gtm_lead_id uuid,
    p_agent_id text,
    p_kind text,
    p_body text,
    p_details jsonb
) returns uuid language plpgsql as $$
declare
    v_activity_id uuid;
begin
    insert into gtm_lead_activity (
        gtm_lead_id,
        actor_user_id,
        agent_id,
        kind,
        body,
        details
    )
    values (
        p_gtm_lead_id,
        p_actor_user_id,
        nullif(trim(p_agent_id), ''),
        p_kind,
        nullif(p_body, ''),
        coalesce(p_details, '{}'::jsonb)
    )
    returning gtm_lead_activity_id into v_activity_id;

    return v_activity_id;
end;
$$;
