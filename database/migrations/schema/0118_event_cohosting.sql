-- Co-hosted events let a primary GOUP group promote an event through other
-- groups without duplicating the event or changing its canonical URL.
create table if not exists event_cohost (
    event_cohost_id uuid primary key default gen_random_uuid(),
    event_id uuid not null references event (event_id) on delete cascade,
    cohost_group_id uuid not null references "group" (group_id) on delete cascade,
    requested_by_user_id uuid not null references "user" (user_id),
    requested_at timestamptz not null default current_timestamp,
    decided_by_user_id uuid references "user" (user_id),
    decided_at timestamptz,
    status text not null default 'pending',
    message text,
    check (status in ('pending', 'approved', 'rejected', 'revoked')),
    check (
        (status = 'pending' and decided_by_user_id is null and decided_at is null)
        or (status <> 'pending' and decided_by_user_id is not null and decided_at is not null)
    ),
    check (message is null or btrim(message) <> '')
);

-- Keep request history while ensuring an event can have at most one active
-- invitation for a co-host group.
create unique index if not exists event_cohost_active_unique_idx
    on event_cohost (event_id, cohost_group_id)
    where status in ('pending', 'approved');
create index if not exists event_cohost_inbox_idx
    on event_cohost (cohost_group_id, status, requested_at desc);
create index if not exists event_cohost_event_idx
    on event_cohost (event_id, status);

-- An explicit per-group preference is needed because a member may want normal
-- announcements from their chapter while declining promoted co-hosted events.
create table if not exists group_cohost_notification_preference (
    group_id uuid not null references "group" (group_id) on delete cascade,
    user_id uuid not null references "user" (user_id) on delete cascade,
    enabled boolean not null default true,
    updated_at timestamptz not null default current_timestamp,
    primary key (group_id, user_id)
);

-- This is the durable idempotency ledger used by asynchronous notification and
-- calendar-invite workers. A retry can safely call claim_event_cohost_delivery.
create table if not exists event_cohost_delivery (
    event_id uuid not null references event (event_id) on delete cascade,
    cohost_group_id uuid not null references "group" (group_id) on delete cascade,
    user_id uuid not null references "user" (user_id) on delete cascade,
    delivery_kind text not null,
    claimed_at timestamptz not null default current_timestamp,
    primary key (event_id, cohost_group_id, user_id, delivery_kind),
    check (delivery_kind in ('event-published', 'calendar-invite', 'event-rescheduled', 'event-canceled'))
);

-- Lists active groups near a primary event/group. p_radius_km is optional and
-- intentionally never hides same-alliance groups merely because their location
-- was not geocoded; the UI can still offer those groups as a manual selection.
create or replace function list_event_cohost_candidates(
    p_actor_user_id uuid,
    p_event_id uuid,
    p_radius_km numeric default 100
)
returns json as $$
    with primary_event as (
        select e.event_id, e.group_id, e.location, g.alliance_id
        from event e
        join "group" g using (group_id)
        where e.event_id = p_event_id
          and e.deleted = false
    ),
    authorized as (
        select 1
        from primary_event pe
        where user_has_group_permission(pe.alliance_id, pe.group_id, p_actor_user_id, 'group.events.write')
    )
    select coalesce(json_agg(json_build_object(
        'alliance_id', g.alliance_id,
        'alliance_name', a.name,
        'distance_km', case
            when pe.location is null or g.location is null then null
            else round((st_distance(pe.location, g.location) / 1000.0)::numeric, 2)
        end,
        'group_id', g.group_id,
        'group_name', g.name,
        'group_slug', coalesce(g.slug_pretty, g.slug),
        'same_alliance', g.alliance_id = pe.alliance_id
    ) order by
        (g.alliance_id = pe.alliance_id) desc,
        st_distance(pe.location, g.location) nulls last,
        g.name,
        g.group_id), '[]'::json)
    from primary_event pe
    cross join authorized
    join "group" g on g.group_id <> pe.group_id
    join alliance a on a.alliance_id = g.alliance_id
    where g.active = true
      and g.deleted = false
      and (
          g.alliance_id = pe.alliance_id
          or (pe.location is not null and g.location is not null
              and st_dwithin(pe.location, g.location, greatest(p_radius_km, 0) * 1000))
      );
$$ language sql stable;

-- Creates an invitation. A user who can manage both groups may self-approve;
-- otherwise target organizers must explicitly approve it, including across
-- alliances. This preserves GOUP's existing alliance terminology and roles.
create or replace function request_event_cohost(
    p_actor_user_id uuid,
    p_event_id uuid,
    p_cohost_group_id uuid,
    p_message text default null
)
returns uuid as $$
declare
    v_primary_group_id uuid;
    v_primary_alliance_id uuid;
    v_cohost_alliance_id uuid;
    v_event_cohost_id uuid;
    v_auto_approve boolean;
begin
    select e.group_id, g.alliance_id
    into v_primary_group_id, v_primary_alliance_id
    from event e
    join "group" g using (group_id)
    where e.event_id = p_event_id
      and e.deleted = false;

    if v_primary_group_id is null then
        raise exception 'event not found';
    end if;
    if v_primary_group_id = p_cohost_group_id then
        raise exception 'a group cannot co-host its own event';
    end if;
    if not user_has_group_permission(v_primary_alliance_id, v_primary_group_id, p_actor_user_id, 'group.events.write') then
        raise exception 'not allowed to co-host this event';
    end if;

    select alliance_id into v_cohost_alliance_id
    from "group"
    where group_id = p_cohost_group_id
      and active = true
      and deleted = false;
    if v_cohost_alliance_id is null then
        raise exception 'co-host group not found or inactive';
    end if;

    select event_cohost_id into v_event_cohost_id
    from event_cohost
    where event_id = p_event_id
      and cohost_group_id = p_cohost_group_id
      and status in ('pending', 'approved');
    if v_event_cohost_id is not null then
        return v_event_cohost_id;
    end if;

    v_auto_approve := user_has_group_permission(
        v_cohost_alliance_id, p_cohost_group_id, p_actor_user_id, 'group.events.write'
    );
    insert into event_cohost (
        event_id, cohost_group_id, requested_by_user_id, message, status,
        decided_by_user_id, decided_at
    )
    values (
        p_event_id, p_cohost_group_id, p_actor_user_id, nullif(btrim(p_message), ''),
        case when v_auto_approve then 'approved' else 'pending' end,
        case when v_auto_approve then p_actor_user_id else null end,
        case when v_auto_approve then current_timestamp else null end
    )
    returning event_cohost_id into v_event_cohost_id;

    return v_event_cohost_id;
end;
$$ language plpgsql;

create or replace function decide_event_cohost(
    p_actor_user_id uuid,
    p_event_cohost_id uuid,
    p_approve boolean
)
returns void as $$
declare
    v_cohost_group_id uuid;
    v_alliance_id uuid;
begin
    select ec.cohost_group_id, g.alliance_id
    into v_cohost_group_id, v_alliance_id
    from event_cohost ec
    join "group" g on g.group_id = ec.cohost_group_id
    where ec.event_cohost_id = p_event_cohost_id
      and ec.status = 'pending'
    for update of ec;
    if v_cohost_group_id is null then
        raise exception 'co-host invitation not found or already decided';
    end if;
    if not user_has_group_permission(v_alliance_id, v_cohost_group_id, p_actor_user_id, 'group.events.write') then
        raise exception 'not allowed to decide this co-host invitation';
    end if;

    update event_cohost
    set status = case when p_approve then 'approved' else 'rejected' end,
        decided_by_user_id = p_actor_user_id,
        decided_at = current_timestamp
    where event_cohost_id = p_event_cohost_id;
end;
$$ language plpgsql;

create or replace function revoke_event_cohost(
    p_actor_user_id uuid,
    p_event_cohost_id uuid
)
returns void as $$
declare
    v_primary_group_id uuid;
    v_primary_alliance_id uuid;
    v_cohost_group_id uuid;
    v_cohost_alliance_id uuid;
begin
    select e.group_id, pg.alliance_id, ec.cohost_group_id, cg.alliance_id
    into v_primary_group_id, v_primary_alliance_id, v_cohost_group_id, v_cohost_alliance_id
    from event_cohost ec
    join event e using (event_id)
    join "group" pg on pg.group_id = e.group_id
    join "group" cg on cg.group_id = ec.cohost_group_id
    where ec.event_cohost_id = p_event_cohost_id
      and ec.status in ('pending', 'approved')
    for update of ec;
    if v_primary_group_id is null then
        raise exception 'co-host invitation not found or inactive';
    end if;
    if not (
        user_has_group_permission(v_primary_alliance_id, v_primary_group_id, p_actor_user_id, 'group.events.write')
        or user_has_group_permission(v_cohost_alliance_id, v_cohost_group_id, p_actor_user_id, 'group.events.write')
    ) then
        raise exception 'not allowed to revoke this co-host invitation';
    end if;

    update event_cohost
    set status = 'revoked', decided_by_user_id = p_actor_user_id, decided_at = current_timestamp
    where event_cohost_id = p_event_cohost_id;
end;
$$ language plpgsql;

create or replace function get_event_cohost_inbox(
    p_actor_user_id uuid,
    p_cohost_group_id uuid
)
returns json as $$
    select coalesce(json_agg(json_build_object(
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
    ) order by ec.requested_at desc), '[]'::json)
    from event_cohost ec
    join event e using (event_id)
    join "group" pg on pg.group_id = e.group_id
    join alliance pa on pa.alliance_id = pg.alliance_id
    join "group" cg on cg.group_id = ec.cohost_group_id
    where ec.cohost_group_id = p_cohost_group_id
      and ec.status = 'pending'
      and user_has_group_permission(cg.alliance_id, cg.group_id, p_actor_user_id, 'group.events.write');
$$ language sql stable;

create or replace function get_event_cohosts(p_event_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'alliance_name', a.name,
        'group_id', g.group_id,
        'group_name', g.name,
        'group_slug', coalesce(g.slug_pretty, g.slug)
    ) order by g.name, g.group_id), '[]'::json)
    from event_cohost ec
    join "group" g on g.group_id = ec.cohost_group_id
    join alliance a on a.alliance_id = g.alliance_id
    where ec.event_id = p_event_id
      and ec.status = 'approved'
      and g.active = true
      and g.deleted = false;
$$ language sql stable;

create or replace function set_group_cohost_notification_preference(
    p_user_id uuid,
    p_group_id uuid,
    p_enabled boolean
)
returns void as $$
begin
    if not exists (
        select 1 from group_member
        where user_id = p_user_id and group_id = p_group_id
    ) then
        raise exception 'user is not a member of this group';
    end if;
    insert into group_cohost_notification_preference (group_id, user_id, enabled)
    values (p_group_id, p_user_id, p_enabled)
    on conflict (group_id, user_id) do update
    set enabled = excluded.enabled, updated_at = current_timestamp;
end;
$$ language plpgsql;

-- This function is the canonical promoted-feed predicate. All calendar/feed
-- consumers should use it instead of copying status logic.
create or replace function event_is_visible_to_group(p_event_id uuid, p_group_id uuid)
returns boolean as $$
    select exists (
        select 1
        from event e
        where e.event_id = p_event_id
          and e.group_id = p_group_id
          and e.deleted = false
    ) or exists (
        select 1
        from event_cohost ec
        join event e using (event_id)
        join "group" cg on cg.group_id = ec.cohost_group_id
        where ec.event_id = p_event_id
          and ec.cohost_group_id = p_group_id
          and ec.status = 'approved'
          and e.deleted = false
          and cg.active = true
          and cg.deleted = false
    );
$$ language sql stable;

-- Preserve the existing public group-events API while making approved
-- co-hosted events appear in that group's standard upcoming feed/calendar.
-- Event summaries remain rooted in the primary host, which keeps event links
-- canonical and supplies public "hosted by" attribution to consumers.
create or replace function get_group_upcoming_events(
    p_alliance_id uuid,
    p_group_slug text,
    p_event_kind_ids text[],
    p_limit int
) returns json as $$
    with target_group as (
        select group_id
        from "group"
        where alliance_id = p_alliance_id
          and (slug = p_group_slug or slug_pretty = p_group_slug)
          and active = true
          and deleted = false
    )
    select coalesce(json_agg(
        get_event_summary(events.primary_alliance_id, events.group_id, events.event_id)
        order by events.starts_at asc, events.event_id asc
    ), '[]')
    from (
        select e.event_id, e.group_id, pg.alliance_id as primary_alliance_id, e.starts_at
        from event e
        join "group" pg using (group_id)
        join target_group tg on event_is_visible_to_group(e.event_id, tg.group_id)
        where e.published = true
          and e.test_event = false
          and e.event_kind_id = any(p_event_kind_ids)
          and e.starts_at is not null
          and e.starts_at > now()
          and e.canceled = false
        order by e.starts_at asc, e.event_id asc
        limit greatest(p_limit, 0)
    ) events;
$$ language sql stable;

create or replace function list_group_cohosted_upcoming_events(
    p_alliance_id uuid,
    p_group_id uuid,
    p_limit integer default 20
)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'event', get_event_summary(e.primary_alliance_id, e.group_id, e.event_id),
        'is_primary_host', e.group_id = p_group_id,
        'primary_group', json_build_object(
            'group_id', pg.group_id,
            'name', pg.name,
            'slug', coalesce(pg.slug_pretty, pg.slug)
        )
    ) order by e.starts_at, e.event_id), '[]'::json)
    from (
        select e.event_id, e.group_id, e.starts_at, pg.alliance_id as primary_alliance_id
        from event e
        join "group" pg on pg.group_id = e.group_id
        join "group" visible_group on visible_group.group_id = p_group_id
        where visible_group.alliance_id = p_alliance_id
          and visible_group.active = true
          and visible_group.deleted = false
          and event_is_visible_to_group(e.event_id, p_group_id)
          and e.published = true
          and e.test_event = false
          and e.canceled = false
          and e.starts_at > current_timestamp
        order by e.starts_at, e.event_id
        limit greatest(p_limit, 0)
    ) e
    join "group" pg on pg.group_id = e.group_id;
$$ language sql stable;

-- Returns eligible recipients only; it respects both global optional-mail
-- preferences and the new group-level co-host preference. Distinct plus the
-- delivery claim below prevent people in multiple participating groups from
-- receiving duplicate calendar invitations.
create or replace function list_event_cohost_notification_recipients(p_event_id uuid)
returns table (cohost_group_id uuid, user_id uuid) as $$
    select distinct ec.cohost_group_id, member.user_id
    from event_cohost ec
    join "group" g on g.group_id = ec.cohost_group_id
    join lateral (
        select gm.user_id from group_member gm where gm.group_id = ec.cohost_group_id
        union
        select gt.user_id from group_team gt
        where gt.group_id = ec.cohost_group_id and gt.accepted = true
    ) member on true
    join "user" u on u.user_id = member.user_id
    left join group_cohost_notification_preference pref
      on pref.group_id = ec.cohost_group_id and pref.user_id = member.user_id
    where ec.event_id = p_event_id
      and ec.status = 'approved'
      and g.active = true
      and g.deleted = false
      and u.optional_notifications_enabled = true
      and coalesce(pref.enabled, true) = true;
$$ language sql stable;

create or replace function claim_event_cohost_delivery(
    p_event_id uuid,
    p_cohost_group_id uuid,
    p_user_id uuid,
    p_delivery_kind text
)
returns boolean as $$
    with eligible as (
        select 1
        from list_event_cohost_notification_recipients(p_event_id) r
        where r.cohost_group_id = p_cohost_group_id
          and r.user_id = p_user_id
    ),
    claimed as (
        insert into event_cohost_delivery (event_id, cohost_group_id, user_id, delivery_kind)
        select p_event_id, p_cohost_group_id, p_user_id, p_delivery_kind
        from eligible
        on conflict do nothing
        returning 1
    )
    select exists (select 1 from claimed);
$$ language sql;
