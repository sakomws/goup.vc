create or replace function update_gtm_lead(
    p_actor_user_id uuid,
    p_alliance_id uuid,
    p_gtm_lead_id uuid,
    p_input jsonb
) returns void language plpgsql as $$
declare
    v_group_id uuid;
begin
    if not exists (
        select 1 from gtm_lead
        where gtm_lead_id = p_gtm_lead_id
          and alliance_id = p_alliance_id
    ) then
        raise exception 'gtm lead not found';
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

    update gtm_lead
    set
        group_id = coalesce(v_group_id, group_id),
        kind = coalesce(nullif(trim(p_input->>'kind'), ''), kind),
        name = coalesce(nullif(trim(p_input->>'name'), ''), name),
        org_name = case
            when p_input ? 'org_name' then nullif(trim(p_input->>'org_name'), '')
            else org_name
        end,
        email = case
            when p_input ? 'email' then nullif(trim(p_input->>'email'), '')
            else email
        end,
        website_url = case
            when p_input ? 'website_url' then nullif(trim(p_input->>'website_url'), '')
            else website_url
        end,
        linkedin_url = case
            when p_input ? 'linkedin_url' then nullif(trim(p_input->>'linkedin_url'), '')
            else linkedin_url
        end,
        owner_user_id = case
            when p_input ? 'owner_user_id' then nullif(p_input->>'owner_user_id', '')::uuid
            else owner_user_id
        end,
        score = case
            when p_input ? 'score' then nullif(p_input->>'score', '')::int
            else score
        end,
        estimated_value_cents = case
            when p_input ? 'estimated_value_cents' then nullif(p_input->>'estimated_value_cents', '')::bigint
            else estimated_value_cents
        end,
        currency = case
            when p_input ? 'currency' then nullif(trim(p_input->>'currency'), '')
            else currency
        end,
        next_action_at = case
            when p_input ? 'next_action_at' and nullif(p_input->>'next_action_at', '') is not null
                then to_timestamp((p_input->>'next_action_at')::bigint)
            when p_input ? 'next_action_at' then null
            else next_action_at
        end,
        renewal_at = case
            when p_input ? 'renewal_at' and nullif(p_input->>'renewal_at', '') is not null
                then to_timestamp((p_input->>'renewal_at')::bigint)
            when p_input ? 'renewal_at' then null
            else renewal_at
        end,
        lost_reason = case
            when p_input ? 'lost_reason' then nullif(trim(p_input->>'lost_reason'), '')
            else lost_reason
        end,
        notes = case
            when p_input ? 'notes' then nullif(trim(p_input->>'notes'), '')
            else notes
        end,
        payload = case
            when p_input ? 'payload' then coalesce(p_input->'payload', '{}'::jsonb)
            else payload
        end,
        updated_at = current_timestamp
    where gtm_lead_id = p_gtm_lead_id
      and alliance_id = p_alliance_id;

    perform insert_audit_log(
        'gtm_lead_updated',
        p_actor_user_id,
        'gtm_lead',
        p_gtm_lead_id,
        p_alliance_id,
        (select group_id from gtm_lead where gtm_lead_id = p_gtm_lead_id)
    );
end;
$$;
