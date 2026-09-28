-- Lists pending co-host invitations across every group the user can manage.
create or replace function list_user_event_cohost_invitations(p_user_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'cohost_group_id', cg.group_id,
        'cohost_group_name', cg.name,
        'event_cohost_id', ec.event_cohost_id,
        'event_id', e.event_id,
        'event_name', e.name,
        'event_slug', e.slug,
        'event_starts_at', floor(extract(epoch from e.starts_at)),
        'message', ec.message,
        'primary_alliance_name', pa.name,
        'primary_group_id', pg.group_id,
        'primary_group_name', pg.name,
        'primary_group_slug', coalesce(pg.slug_pretty, pg.slug),
        'requested_at', floor(extract(epoch from ec.requested_at))
    ) order by ec.requested_at desc, ec.event_cohost_id), '[]'::json)
    from event_cohost ec
    join event e using (event_id)
    join "group" pg on pg.group_id = e.group_id
    join alliance pa on pa.alliance_id = pg.alliance_id
    join "group" cg on cg.group_id = ec.cohost_group_id
    where ec.status = 'pending'
      and e.deleted = false
      and cg.active = true
      and cg.deleted = false
      and user_has_group_permission(
          cg.alliance_id,
          cg.group_id,
          p_user_id,
          'group.events.write'
      );
$$ language sql stable;
