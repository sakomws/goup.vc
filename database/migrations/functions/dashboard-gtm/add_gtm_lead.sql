create or replace function add_gtm_lead(
    p_actor_user_id uuid,
    p_alliance_id uuid,
    p_input jsonb
) returns uuid language plpgsql as $$
declare
    v_lead_id uuid;
    v_group_id uuid;
    v_name text := trim(p_input->>'name');
    v_email text := nullif(trim(p_input->>'email'), '');
    v_website text := nullif(trim(p_input->>'website_url'), '');
begin
    if v_name is null or v_name = '' then
        raise exception 'gtm lead name is required';
    end if;

    v_group_id := nullif(p_input->>'group_id', '')::uuid;
    if v_group_id is not null then
        if not exists (
            select 1 from "group"
            where group_id = v_group_id
              and alliance_id = p_alliance_id
              and deleted = false
        ) then
            raise exception 'group does not belong to alliance';
        end if;
    end if;

    insert into gtm_lead (
        alliance_id,
        group_id,
        kind,
        stage,
        name,
        org_name,
        email,
        website_url,
        linkedin_url,
        landscape_entry_id,
        user_id,
        owner_user_id,
        score,
        estimated_value_cents,
        currency,
        next_action_at,
        renewal_at,
        source,
        notes,
        payload
    )
    values (
        p_alliance_id,
        v_group_id,
        trim(p_input->>'kind'),
        coalesce(nullif(trim(p_input->>'stage'), ''), 'lead_generation'),
        v_name,
        nullif(trim(p_input->>'org_name'), ''),
        v_email,
        v_website,
        nullif(trim(p_input->>'linkedin_url'), ''),
        nullif(p_input->>'landscape_entry_id', '')::uuid,
        nullif(p_input->>'user_id', '')::uuid,
        coalesce(nullif(p_input->>'owner_user_id', '')::uuid, p_actor_user_id),
        nullif(p_input->>'score', '')::int,
        nullif(p_input->>'estimated_value_cents', '')::bigint,
        nullif(trim(p_input->>'currency'), ''),
        case
            when nullif(p_input->>'next_action_at', '') is null then null
            else to_timestamp((p_input->>'next_action_at')::bigint)
        end,
        case
            when nullif(p_input->>'renewal_at', '') is null then null
            else to_timestamp((p_input->>'renewal_at')::bigint)
        end,
        coalesce(nullif(trim(p_input->>'source'), ''), 'manual'),
        nullif(trim(p_input->>'notes'), ''),
        coalesce(p_input->'payload', '{}'::jsonb)
    )
    returning gtm_lead_id into v_lead_id;

    perform add_gtm_lead_activity(
        p_actor_user_id,
        v_lead_id,
        null,
        'stage_change',
        'Lead created',
        jsonb_build_object('stage', coalesce(nullif(trim(p_input->>'stage'), ''), 'lead_generation'))
    );

    perform insert_audit_log(
        'gtm_lead_added',
        p_actor_user_id,
        'gtm_lead',
        v_lead_id,
        p_alliance_id,
        v_group_id
    );

    return v_lead_id;
exception
    when unique_violation then
        raise exception 'gtm lead already exists for this email or website';
end;
$$;
