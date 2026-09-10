create or replace function review_gtm_agent_draft(
    p_actor_user_id uuid,
    p_alliance_id uuid,
    p_draft_id uuid,
    p_input jsonb
) returns jsonb language plpgsql as $$
declare
    v_draft gtm_agent_draft;
    v_status text := trim(p_input->>'status');
    v_body text;
    v_suggested_stage text;
    v_payload jsonb;
    v_created_ids uuid[] := '{}';
    v_candidate jsonb;
    v_lead_id uuid;
    v_side_effects jsonb := '{}'::jsonb;
begin
    select * into v_draft
    from gtm_agent_draft
    where gtm_agent_draft_id = p_draft_id
      and alliance_id = p_alliance_id;

    if not found then
        raise exception 'gtm agent draft not found';
    end if;
    if v_draft.status <> 'pending' then
        raise exception 'gtm agent draft is not pending';
    end if;
    if v_status not in ('approved', 'rejected') then
        raise exception 'gtm agent draft review status must be approved or rejected';
    end if;

    v_body := coalesce(p_input->>'body', v_draft.body);
    v_suggested_stage := coalesce(
        nullif(trim(p_input->>'suggested_stage'), ''),
        v_draft.suggested_stage
    );
    v_payload := v_draft.payload;
    if p_input ? 'payload' then
        v_payload := v_payload || coalesce(p_input->'payload', '{}'::jsonb);
    end if;

    update gtm_agent_draft
    set
        status = v_status,
        body = v_body,
        suggested_stage = v_suggested_stage,
        payload = v_payload,
        reviewed_at = current_timestamp,
        reviewed_by = p_actor_user_id
    where gtm_agent_draft_id = p_draft_id;

    if v_status = 'rejected' then
        if v_draft.gtm_lead_id is not null then
            perform add_gtm_lead_activity(
                p_actor_user_id,
                v_draft.gtm_lead_id,
                v_draft.agent_id,
                'draft_rejected',
                coalesce(nullif(trim(p_input->>'note'), ''), v_draft.title),
                jsonb_build_object('gtm_agent_draft_id', p_draft_id)
            );
        end if;
        return jsonb_build_object(
            'gtm_agent_draft_id', p_draft_id,
            'status', v_status,
            'created_lead_ids', '[]'::jsonb
        );
    end if;

    if v_draft.agent_id = 'lead_generation' and v_payload ? 'candidates' then
        for v_candidate in select * from jsonb_array_elements(v_payload->'candidates')
        loop
            begin
                v_lead_id := add_gtm_lead(
                    p_actor_user_id,
                    p_alliance_id,
                    v_candidate || jsonb_build_object(
                        'stage', 'lead_generation',
                        'source', coalesce(v_candidate->>'source', 'discovery')
                    )
                );
                v_created_ids := array_append(v_created_ids, v_lead_id);
            exception
                when others then
                    continue;
            end;
        end loop;
    elsif v_draft.gtm_lead_id is not null then
        perform update_gtm_lead(
            p_actor_user_id,
            p_alliance_id,
            v_draft.gtm_lead_id,
            v_payload
        );

        if v_suggested_stage is not null then
            perform transition_gtm_lead(
                p_actor_user_id,
                p_alliance_id,
                v_draft.gtm_lead_id,
                v_suggested_stage,
                true,
                jsonb_build_object(
                    'agent_id', v_draft.agent_id,
                    'body', v_body,
                    'payload', v_payload
                )
            );
        end if;

        if v_suggested_stage = 'won' and v_payload ? 'side_effects' then
            v_side_effects := apply_gtm_won_side_effects(
                p_actor_user_id,
                v_draft.gtm_lead_id,
                v_payload->'side_effects'
            );
        end if;

        perform add_gtm_lead_activity(
            p_actor_user_id,
            v_draft.gtm_lead_id,
            v_draft.agent_id,
            'draft_approved',
            v_draft.title,
            jsonb_build_object(
                'gtm_agent_draft_id', p_draft_id,
                'suggested_stage', v_suggested_stage
            )
        );
    end if;

    return jsonb_build_object(
        'gtm_agent_draft_id', p_draft_id,
        'status', v_status,
        'created_lead_ids', to_jsonb(v_created_ids),
        'side_effects', v_side_effects,
        'gtm_lead_id', v_draft.gtm_lead_id,
        'agent_id', v_draft.agent_id,
        'body', v_body,
        'suggested_stage', v_suggested_stage
    );
end;
$$;
