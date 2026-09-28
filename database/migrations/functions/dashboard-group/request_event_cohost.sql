-- Creates a co-host invitation, or returns the existing active request.
create or replace function request_event_cohost(
    p_actor_user_id uuid,
    p_event_id uuid,
    p_cohost_group_id uuid,
    p_message text default null
)
returns uuid as $$
declare
    v_primary_group_id uuid;
    v_primary_alliance_id uuid;
    v_cohost_alliance_id uuid;
    v_event_cohost_id uuid;
    v_auto_approve boolean;
begin
    select e.group_id, g.alliance_id
    into v_primary_group_id, v_primary_alliance_id
    from event e
    join "group" g using (group_id)
    where e.event_id = p_event_id
      and e.deleted = false;

    if v_primary_group_id is null then
        raise exception 'event not found';
    end if;
    if v_primary_group_id = p_cohost_group_id then
        raise exception 'a group cannot co-host its own event';
    end if;
    if not user_has_group_permission(
        v_primary_alliance_id,
        v_primary_group_id,
        p_actor_user_id,
        'group.events.write'
    ) then
        raise exception 'not allowed to co-host this event';
    end if;

    select alliance_id into v_cohost_alliance_id
    from "group"
    where group_id = p_cohost_group_id
      and active = true
      and deleted = false;
    if v_cohost_alliance_id is null then
        raise exception 'co-host group not found or inactive';
    end if;

    v_auto_approve := user_has_group_permission(
        v_cohost_alliance_id,
        p_cohost_group_id,
        p_actor_user_id,
        'group.events.write'
    );

    insert into event_cohost (
        event_id,
        cohost_group_id,
        requested_by_user_id,
        message,
        status,
        decided_by_user_id,
        decided_at
    )
    values (
        p_event_id,
        p_cohost_group_id,
        p_actor_user_id,
        nullif(btrim(p_message), ''),
        case when v_auto_approve then 'approved' else 'pending' end,
        case when v_auto_approve then p_actor_user_id else null end,
        case when v_auto_approve then current_timestamp else null end
    )
    on conflict (event_id, cohost_group_id)
        where status in ('pending', 'approved')
    do nothing
    returning event_cohost_id into v_event_cohost_id;

    if v_event_cohost_id is null then
        select event_cohost_id into v_event_cohost_id
        from event_cohost
        where event_id = p_event_id
          and cohost_group_id = p_cohost_group_id
          and status in ('pending', 'approved');
    end if;

    return v_event_cohost_id;
end;
$$ language plpgsql;
