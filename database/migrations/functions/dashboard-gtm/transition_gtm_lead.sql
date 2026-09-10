create or replace function transition_gtm_lead(
    p_actor_user_id uuid,
    p_alliance_id uuid,
    p_gtm_lead_id uuid,
    p_stage text,
    p_human boolean,
    p_details jsonb
) returns void language plpgsql as $$
declare
    v_lead gtm_lead;
    v_stage text := trim(p_stage);
begin
    select * into v_lead
    from gtm_lead
    where gtm_lead_id = p_gtm_lead_id
      and alliance_id = p_alliance_id;

    if not found then
        raise exception 'gtm lead not found';
    end if;

    if not gtm_is_legal_transition(v_lead.stage, v_stage, coalesce(p_human, true)) then
        raise exception 'illegal gtm stage transition from % to %', v_lead.stage, v_stage;
    end if;

    update gtm_lead
    set
        stage = v_stage,
        lost_reason = case
            when v_stage = 'lost' then coalesce(
                nullif(trim(p_details->>'lost_reason'), ''),
                lost_reason
            )
            when v_stage <> 'lost' then lost_reason
            else lost_reason
        end,
        score = coalesce(nullif(p_details->>'score', '')::int, score),
        payload = case
            when p_details ? 'payload' then coalesce(p_details->'payload', payload)
            else payload
        end,
        updated_at = current_timestamp
    where gtm_lead_id = p_gtm_lead_id;

    perform add_gtm_lead_activity(
        p_actor_user_id,
        p_gtm_lead_id,
        nullif(trim(p_details->>'agent_id'), ''),
        'stage_change',
        coalesce(nullif(trim(p_details->>'body'), ''), format('Stage moved to %s', v_stage)),
        jsonb_build_object(
            'from', v_lead.stage,
            'to', v_stage,
            'human', coalesce(p_human, true)
        ) || coalesce(p_details, '{}'::jsonb)
    );

    perform insert_audit_log(
        'gtm_lead_transitioned',
        p_actor_user_id,
        'gtm_lead',
        p_gtm_lead_id,
        p_alliance_id,
        v_lead.group_id,
        null,
        jsonb_build_object('from', v_lead.stage, 'to', v_stage)
    );
end;
$$;
