-- move_event moves an event to another group within the same alliance.
create or replace function move_event(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_target_group_id uuid
)
returns void as $$
declare
    v_source_alliance_id uuid;
    v_target_alliance_id uuid;
    v_slug text;
begin
    -- Load and lock the event, ensuring it belongs to the source group and is active
    select e.slug, g.alliance_id
    into v_slug, v_source_alliance_id
    from event e
    join "group" g on g.group_id = e.group_id
    where e.event_id = p_event_id
      and e.group_id = p_group_id
      and e.deleted = false
      and e.canceled = false
    for update of e;

    if not found then
        raise exception 'event not found or inactive';
    end if;

    -- Moving an event changes alliance-level group ownership. Group-local
    -- event managers must not be able to transfer events between groups.
    if not user_has_alliance_permission(
        v_source_alliance_id,
        p_actor_user_id,
        'alliance.groups.write'::text
    ) then
        raise exception 'alliance group management permission required';
    end if;

    -- Nothing to do when the event already belongs to the target group
    if p_group_id = p_target_group_id then
        return;
    end if;

    -- The target group must exist and be active
    select g.alliance_id
    into v_target_alliance_id
    from "group" g
    where g.group_id = p_target_group_id
      and g.deleted = false
      and g.active = true;

    if not found then
        raise exception 'target group not found or inactive';
    end if;

    -- Events can only move within the same alliance so category and alliance data stay valid
    if v_target_alliance_id <> v_source_alliance_id then
        raise exception 'events can only be moved between groups in the same alliance';
    end if;

    -- Enforce the unique (slug, group_id) constraint before moving
    if exists (
        select 1
        from event
        where group_id = p_target_group_id
          and slug = v_slug
          and event_id <> p_event_id
    ) then
        raise exception 'an event with the same slug already exists in the target group';
    end if;

    -- Move the event to the target group
    update event
    set group_id = p_target_group_id
    where event_id = p_event_id
      and group_id = p_group_id
      and deleted = false
      and canceled = false;

    -- Reusable sponsors stay with their original group. Event-only sponsors
    -- move with the event and remain scoped to it.
    delete from event_sponsor es
    using group_sponsor gs
    where es.event_id = p_event_id
      and es.group_sponsor_id = gs.group_sponsor_id
      and gs.event_id is null;

    update group_sponsor
    set group_id = p_target_group_id
    where event_id = p_event_id;

    -- Record the move against the target group
    perform insert_audit_log(
        'event_moved',
        p_actor_user_id,
        'event',
        p_event_id,
        v_target_alliance_id,
        p_target_group_id,
        p_event_id
    );
end;
$$ language plpgsql;
