-- Returns the display data needed for a co-host invitation email.
create or replace function get_event_cohost_notification_data(
    p_event_cohost_id uuid
)
returns json as $$
    select json_build_object(
        'cohost_group_name', cg.name,
        'event_name', e.name,
        'message', ec.message,
        'primary_group_name', pg.name
    )
    from event_cohost ec
    join event e using (event_id)
    join "group" pg on pg.group_id = e.group_id
    join "group" cg on cg.group_id = ec.cohost_group_id
    where ec.event_cohost_id = p_event_cohost_id;
$$ language sql stable;
