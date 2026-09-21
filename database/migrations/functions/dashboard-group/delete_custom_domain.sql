-- Deletes the custom domain assigned to a group or one of its events.
create or replace function delete_custom_domain(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid
)
returns boolean
language plpgsql
as $$
declare
    v_alliance_id uuid;
    v_custom_domain_id uuid;
    v_hostname text;
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

    delete from custom_domain
    where (
        p_event_id is null
        and group_id = p_group_id
    ) or (
        p_event_id is not null
        and event_id = p_event_id
    )
    returning custom_domain_id, hostname
    into v_custom_domain_id, v_hostname;

    if not found then
        return false;
    end if;

    perform insert_audit_log(
        'custom_domain_deleted',
        p_actor_user_id,
        'custom_domain',
        v_custom_domain_id,
        v_alliance_id,
        p_group_id,
        p_event_id,
        jsonb_build_object('hostname', v_hostname)
    );

    return true;
end;
$$;
