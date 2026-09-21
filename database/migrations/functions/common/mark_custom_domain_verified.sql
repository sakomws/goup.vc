-- Marks a domain verified only when its current verification token matches.
create or replace function mark_custom_domain_verified(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_custom_domain_id uuid,
    p_hostname text,
    p_verification_token text
)
returns jsonb
language plpgsql
as $$
declare
    v_alliance_id uuid;
    v_custom_domain custom_domain;
    v_permission text;
begin
    select alliance_id
    into v_alliance_id
    from "group"
    where group_id = p_group_id
      and deleted = false;

    if not found then
        raise exception 'group not found';
    end if;

    if p_event_id is not null and not exists (
        select 1
        from event
        where event_id = p_event_id
          and group_id = p_group_id
          and deleted = false
    ) then
        raise exception 'event not found in group';
    end if;

    v_permission := case
        when p_event_id is null then 'group.settings.write'
        else 'group.events.write'
    end;

    if not user_has_group_permission(
        v_alliance_id,
        p_group_id,
        p_actor_user_id,
        v_permission
    ) then
        raise exception 'custom domain permission required';
    end if;

    update custom_domain
    set verified_at = current_timestamp,
        updated_at = current_timestamp
    where custom_domain_id = p_custom_domain_id
      and hostname = normalize_custom_domain_hostname(p_hostname)
      and verification_token = p_verification_token
      and verified_at is null
      and (
          (p_event_id is null and group_id = p_group_id)
          or (p_event_id is not null and event_id = p_event_id)
      )
    returning * into v_custom_domain;

    if not found then
        raise exception 'custom domain changed; retry DNS verification';
    end if;

    return custom_domain_json(v_custom_domain);
end;
$$;
