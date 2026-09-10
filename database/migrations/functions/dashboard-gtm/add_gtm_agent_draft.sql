create or replace function add_gtm_agent_draft(
    p_actor_user_id uuid,
    p_alliance_id uuid,
    p_input jsonb
) returns uuid language plpgsql as $$
declare
    v_draft_id uuid;
    v_lead_id uuid := nullif(p_input->>'gtm_lead_id', '')::uuid;
    v_group_id uuid := nullif(p_input->>'group_id', '')::uuid;
    v_agent_id text := trim(p_input->>'agent_id');
begin
    if v_lead_id is not null then
        select group_id into v_group_id
        from gtm_lead
        where gtm_lead_id = v_lead_id
          and alliance_id = p_alliance_id;
        if not found then
            raise exception 'gtm lead not found';
        end if;

        update gtm_agent_draft
        set status = 'superseded',
            reviewed_at = current_timestamp
        where gtm_lead_id = v_lead_id
          and agent_id = v_agent_id
          and status = 'pending';
    end if;

    insert into gtm_agent_draft (
        alliance_id,
        group_id,
        gtm_lead_id,
        agent_id,
        title,
        body,
        suggested_stage,
        payload
    )
    values (
        p_alliance_id,
        v_group_id,
        v_lead_id,
        v_agent_id,
        trim(p_input->>'title'),
        coalesce(p_input->>'body', ''),
        nullif(trim(p_input->>'suggested_stage'), ''),
        coalesce(p_input->'payload', '{}'::jsonb)
    )
    returning gtm_agent_draft_id into v_draft_id;

    if v_lead_id is not null then
        perform add_gtm_lead_activity(
            p_actor_user_id,
            v_lead_id,
            v_agent_id,
            'draft_created',
            trim(p_input->>'title'),
            jsonb_build_object('gtm_agent_draft_id', v_draft_id)
        );
    end if;

    return v_draft_id;
end;
$$;
