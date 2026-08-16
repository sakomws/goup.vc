-- Returns upcoming events for a specific group.
create or replace function get_group_upcoming_events(
    p_alliance_id uuid,
    p_group_slug text,
    p_event_kind_ids text[],
    p_limit int
) returns json as $$
    -- Events retain their primary host and canonical URL. Approved co-hosts are
    -- additionally visible from the selected group's public feed/calendar.
    with target_group as (
        select group_id
        from "group"
        where alliance_id = p_alliance_id
        and (slug = p_group_slug or slug_pretty = p_group_slug)
        and active = true
        and deleted = false
    )
    select coalesce(json_agg(
        get_event_summary(e.primary_alliance_id, e.group_id, e.event_id)
        order by e.starts_at asc, e.event_id asc
    ), '[]')
    from (
        select e.event_id, e.group_id, pg.alliance_id as primary_alliance_id, e.starts_at
        from event e
        join "group" pg using (group_id)
        join target_group tg on event_is_visible_to_group(e.event_id, tg.group_id)
        where e.deleted = false
        and e.published = true
        and e.test_event = false
        and e.event_kind_id = any(p_event_kind_ids)
        and e.starts_at is not null
        and e.starts_at > now()
        and e.canceled = false
        order by e.starts_at asc, e.event_id asc
        limit p_limit
    ) e;
$$ language sql;
