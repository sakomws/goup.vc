-- Lists groups that can be selected as a parent for the current group.
create or replace function list_group_parent_options(
    p_alliance_id uuid,
    p_group_id uuid
)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'group_id', g.group_id,
        'name', g.name,
        'slug', g.slug
    ) order by g.name, g.slug), '[]'::json)
    from "group" g
    where g.alliance_id = p_alliance_id
    and g.deleted = false
    and g.active = true
    and g.parent_group_id is null
    and g.group_id <> p_group_id
    and not exists (
        select 1
        from "group" child
        where child.parent_group_id = p_group_id
        and child.deleted = false
    );
$$ language sql;
