-- Creates or replaces the single custom domain assigned to a group or event.
create or replace function upsert_custom_domain(
    p_actor_user_id uuid,
    p_alliance_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_hostname text,
    p_verification_token text
)
returns jsonb
language plpgsql
as $$
declare
    v_custom_domain custom_domain;
    v_custom_domain_id uuid;
    v_hostname text;
    v_permission text;
begin
    if not exists (
        select 1
        from "group"
        where group_id = p_group_id
          and alliance_id = p_alliance_id
          and deleted = false
          and active = true
    ) then
        raise exception 'group not found or inactive';
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
        p_alliance_id,
        p_group_id,
        p_actor_user_id,
        v_permission
    ) then
        raise exception 'custom domain permission required';
    end if;

    v_hostname := normalize_custom_domain_hostname(p_hostname);
    if v_hostname is null or v_hostname = '' then
        raise exception 'hostname is required';
    end if;

    if p_verification_token is null or btrim(p_verification_token) = '' then
        raise exception 'verification token is required';
    end if;

    select custom_domain_id
    into v_custom_domain_id
    from custom_domain
    where (
        p_event_id is null
        and group_id = p_group_id
    ) or (
        p_event_id is not null
        and event_id = p_event_id
    )
    for update;

    if found then
        update custom_domain
        set hostname = v_hostname,
            verification_token = p_verification_token,
            verification_token_created_at = current_timestamp,
            verified_at = null,
            activated_at = null,
            updated_at = current_timestamp
        where custom_domain_id = v_custom_domain_id
        returning * into v_custom_domain;
    else
        insert into custom_domain (
            hostname,
            group_id,
            event_id,
            verification_token
        )
        values (
            v_hostname,
            case when p_event_id is null then p_group_id end,
            p_event_id,
            p_verification_token
        )
        returning * into v_custom_domain;
    end if;

    perform insert_audit_log(
        'custom_domain_saved',
        p_actor_user_id,
        'custom_domain',
        v_custom_domain.custom_domain_id,
        p_alliance_id,
        p_group_id,
        p_event_id,
        jsonb_build_object('hostname', v_custom_domain.hostname)
    );

    return custom_domain_json(v_custom_domain);
exception
    when unique_violation then
        raise exception 'hostname is already assigned';
    when check_violation then
        raise exception 'hostname is invalid';
end;
$$;
