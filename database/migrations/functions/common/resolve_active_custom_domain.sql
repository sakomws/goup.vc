-- Resolves an active hostname to exactly one group or event target.
create or replace function resolve_active_custom_domain(
    p_hostname text
)
returns jsonb
stable
language sql
as $$
    select jsonb_strip_nulls(jsonb_build_object(
        'alliance_name', a.name,
        'event_slug', e.slug,
        'group_slug', coalesce(g.slug_pretty, g.slug),
        'hostname', cd.hostname
    ))
    from custom_domain cd
    left join event e on e.event_id = cd.event_id
    join "group" g on g.group_id = coalesce(cd.group_id, e.group_id)
    join alliance a on a.alliance_id = g.alliance_id
    where cd.hostname = normalize_custom_domain_hostname(p_hostname)
      and cd.activated_at is not null
      and g.active = true
      and g.deleted = false
      and (e.event_id is null or e.deleted = false);
$$;
