-- Returns the active, non-deleted groups of an alliance, whose teams the
-- alliance contact feature can reach.
create or replace function alliance_contact_groups(p_alliance_id uuid)
returns table (
    group_category_id uuid,
    group_id uuid,
    parent_group_id uuid,
    region_id uuid
) as $$
    select
        g.group_category_id,
        g.group_id,
        g.parent_group_id,
        g.region_id
    from "group" g
    where g.alliance_id = p_alliance_id
    and g.active = true
    and g.deleted = false;
$$ language sql stable;
