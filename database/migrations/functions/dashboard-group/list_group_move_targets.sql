-- Lists groups an event can be moved into: active groups in the same alliance
-- where the actor can manage events, excluding the event's current group.
create or replace function list_group_move_targets(
    p_actor_user_id uuid,
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
      and g.group_id <> p_group_id
      and user_has_group_permission(p_alliance_id, g.group_id, p_actor_user_id, 'group.events.write'::text);
$$ language sql;
