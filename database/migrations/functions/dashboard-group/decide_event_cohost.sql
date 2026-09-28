-- Decides a co-host invitation. Repeating the same decision is a safe no-op;
-- a conflicting concurrent decision remains first-writer-wins.
create or replace function decide_event_cohost(
    p_actor_user_id uuid,
    p_event_cohost_id uuid,
    p_approve boolean
)
returns void as $$
declare
    v_cohost_group_id uuid;
    v_alliance_id uuid;
    v_status text;
    v_requested_status text := case when p_approve then 'approved' else 'rejected' end;
begin
    select ec.cohost_group_id, g.alliance_id, ec.status
    into v_cohost_group_id, v_alliance_id, v_status
    from event_cohost ec
    join "group" g on g.group_id = ec.cohost_group_id
    where ec.event_cohost_id = p_event_cohost_id
    for update of ec;

    if v_cohost_group_id is null then
        raise exception 'co-host invitation not found';
    end if;
    if not user_has_group_permission(
        v_alliance_id,
        v_cohost_group_id,
        p_actor_user_id,
        'group.events.write'
    ) then
        raise exception 'not allowed to decide this co-host invitation';
    end if;

    if v_status = v_requested_status then
        return;
    end if;
    if v_status <> 'pending' then
        raise exception 'co-host invitation already decided';
    end if;

    update event_cohost
    set status = v_requested_status,
        decided_by_user_id = p_actor_user_id,
        decided_at = current_timestamp
    where event_cohost_id = p_event_cohost_id;
end;
$$ language plpgsql;
