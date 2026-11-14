-- Resolves an event that accepts inbox contact to its owning group. The event
-- page must be publicly available and its owning group part of the alliance.
-- Canceled and past events qualify, and co-host groups are never considered.
create or replace function resolve_inbox_contact_event(p_alliance_id uuid, p_event_id uuid)
returns table (event_id uuid, group_id uuid) as $$
    select
        e.event_id,
        e.group_id
    from event e
    join "group" g on g.group_id = e.group_id
    where e.event_id = p_event_id
    and g.alliance_id = p_alliance_id
    and is_inbox_event_public(e, g);
$$ language sql stable;
