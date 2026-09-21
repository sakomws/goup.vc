-- Returns the custom domain assigned to a group or one of its events.
create or replace function get_custom_domain(
    p_group_id uuid,
    p_event_id uuid
)
returns jsonb
stable
language sql
as $$
    select custom_domain_json(cd)
    from custom_domain cd
    where (
        p_event_id is null
        and cd.group_id = p_group_id
    ) or (
        p_event_id is not null
        and cd.event_id = p_event_id
        and exists (
            select 1
            from event e
            where e.event_id = cd.event_id
              and e.group_id = p_group_id
              and e.deleted = false
        )
    );
$$;
