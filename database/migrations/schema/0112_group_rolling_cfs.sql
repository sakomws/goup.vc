-- A standing Call for Speakers belongs to a group rather than a dated event.
-- Event-bound CFS rows deliberately remain unchanged; assignments create a
-- regular approved event submission so the existing session scheduling flow
-- and its approval trigger continue to apply.
create table group_cfs (
    group_id uuid primary key references "group" (group_id) on delete cascade,
    enabled boolean not null default false,
    description text,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp,
    check (description is null or btrim(description) <> '')
);

create table group_cfs_label (
    group_cfs_label_id uuid primary key default gen_random_uuid(),
    group_id uuid not null references "group" (group_id) on delete cascade,
    name text not null check (btrim(name) <> ''),
    color text not null check (btrim(color) <> ''),
    created_at timestamptz not null default current_timestamp,
    unique (group_id, name)
);

create table group_cfs_submission (
    group_cfs_submission_id uuid primary key default gen_random_uuid(),
    group_id uuid not null references "group" (group_id) on delete cascade,
    session_proposal_id uuid not null references session_proposal (session_proposal_id),
    status_id text not null references cfs_submission_status (cfs_submission_status_id)
        default 'not-reviewed',
    action_required_message text,
    reviewed_by uuid references "user" (user_id),
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz,
    unique (group_id, session_proposal_id),
    check (action_required_message is null or btrim(action_required_message) <> '')
);

create index group_cfs_submission_group_status_idx
    on group_cfs_submission (group_id, status_id, created_at desc);
create index group_cfs_submission_proposal_idx
    on group_cfs_submission (session_proposal_id);

create table group_cfs_submission_label (
    group_cfs_submission_id uuid not null references group_cfs_submission (group_cfs_submission_id) on delete cascade,
    group_cfs_label_id uuid not null references group_cfs_label (group_cfs_label_id) on delete cascade,
    created_at timestamptz not null default current_timestamp,
    primary key (group_cfs_submission_id, group_cfs_label_id)
);

create table group_cfs_submission_rating (
    group_cfs_submission_id uuid not null references group_cfs_submission (group_cfs_submission_id) on delete cascade,
    reviewer_id uuid not null references "user" (user_id),
    stars smallint not null check (stars between 1 and 5),
    comments text,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz,
    primary key (group_cfs_submission_id, reviewer_id),
    check (comments is null or btrim(comments) <> '')
);

-- Assignment history is append-only.  The generated event CFS submission is
-- intentionally retained so an approved proposal may be reused for multiple
-- dated events without bypassing event/session constraints.
create table group_cfs_submission_assignment (
    group_cfs_submission_assignment_id uuid primary key default gen_random_uuid(),
    group_cfs_submission_id uuid not null references group_cfs_submission (group_cfs_submission_id) on delete cascade,
    event_id uuid not null references event (event_id) on delete cascade,
    event_cfs_submission_id uuid not null references cfs_submission (cfs_submission_id) on delete restrict,
    assigned_by uuid references "user" (user_id) on delete set null,
    assigned_at timestamptz not null default current_timestamp,
    unique (group_cfs_submission_id, event_id),
    unique (event_cfs_submission_id)
);

create index group_cfs_submission_assignment_event_idx
    on group_cfs_submission_assignment (event_id, assigned_at desc);

create or replace function update_group_cfs(
    p_group_id uuid,
    p_enabled boolean,
    p_description text,
    p_labels jsonb default '[]'::jsonb
)
returns void as $$
declare
    v_label jsonb;
begin
    if jsonb_typeof(p_labels) <> 'array' or jsonb_array_length(p_labels) > 20 then
        raise exception 'invalid rolling cfs labels';
    end if;

    insert into group_cfs (group_id, enabled, description)
    values (p_group_id, p_enabled, nullif(btrim(p_description), ''))
    on conflict (group_id) do update set
        enabled = excluded.enabled,
        description = excluded.description,
        updated_at = current_timestamp;

    delete from group_cfs_label
    where group_id = p_group_id
    and group_cfs_label_id <> all(coalesce(array(
        select nullif(value->>'event_cfs_label_id', '')::uuid
        from jsonb_array_elements(p_labels)
        where nullif(value->>'event_cfs_label_id', '') is not null
    ), '{}'::uuid[]));
    for v_label in select value from jsonb_array_elements(p_labels)
    loop
        if nullif(btrim(v_label->>'name'), '') is null
            or nullif(btrim(v_label->>'color'), '') is null then
            raise exception 'invalid rolling cfs label';
        end if;
        insert into group_cfs_label (group_cfs_label_id, group_id, name, color)
        values (
            coalesce(nullif(v_label->>'event_cfs_label_id', '')::uuid, gen_random_uuid()),
            p_group_id,
            btrim(v_label->>'name'),
            btrim(v_label->>'color')
        )
        on conflict (group_cfs_label_id) do update set
            name = excluded.name,
            color = excluded.color
        where group_cfs_label.group_id = p_group_id;
    end loop;
end;
$$ language plpgsql;

create or replace function add_group_cfs_submission(
    p_alliance_id uuid,
    p_group_id uuid,
    p_user_id uuid,
    p_session_proposal_id uuid,
    p_label_ids uuid[] default null
)
returns uuid as $$
declare
    v_submission_id uuid;
begin
    if not exists (
        select 1 from "group" g
        join group_cfs gc on gc.group_id = g.group_id and gc.enabled
        where g.group_id = p_group_id and g.alliance_id = p_alliance_id and g.active
    ) then
        raise exception 'rolling cfs is not enabled for this group';
    end if;
    if not exists (
        select 1 from session_proposal
        where session_proposal_id = p_session_proposal_id
        and user_id = p_user_id
        and session_proposal_status_id = 'ready-for-submission'
    ) then
        raise exception 'session proposal not ready for submission';
    end if;
    if cardinality(p_label_ids) > 10 or exists (
        select 1 from unnest(coalesce(p_label_ids, '{}'::uuid[])) label_id
        where not exists (
            select 1 from group_cfs_label
            where group_id = p_group_id and group_cfs_label_id = label_id
        )
    ) then
        raise exception 'invalid rolling cfs labels';
    end if;

    insert into group_cfs_submission (group_id, session_proposal_id)
    values (p_group_id, p_session_proposal_id)
    returning group_cfs_submission_id into v_submission_id;

    insert into group_cfs_submission_label (group_cfs_submission_id, group_cfs_label_id)
    select v_submission_id, label_id from unnest(coalesce(p_label_ids, '{}'::uuid[])) label_id;
    return v_submission_id;
end;
$$ language plpgsql;

create or replace function assign_group_cfs_submission(
    p_reviewer_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_group_cfs_submission_id uuid
)
returns uuid as $$
declare
    v_session_proposal_id uuid;
    v_event_cfs_submission_id uuid;
begin
    select gcs.session_proposal_id into v_session_proposal_id
    from group_cfs_submission gcs
    where gcs.group_cfs_submission_id = p_group_cfs_submission_id
    and gcs.group_id = p_group_id
    and gcs.status_id = 'approved'
    for update;
    if not found then
        raise exception 'approved rolling cfs submission not found';
    end if;
    if not exists (
        select 1 from event
        where event_id = p_event_id and group_id = p_group_id
        and deleted = false and canceled = false
    ) then
        raise exception 'event not found or unavailable';
    end if;

    insert into cfs_submission (event_id, session_proposal_id, status_id, reviewed_by, updated_at)
    values (p_event_id, v_session_proposal_id, 'approved', p_reviewer_id, current_timestamp)
    on conflict (event_id, session_proposal_id) do update
        set status_id = 'approved', reviewed_by = excluded.reviewed_by, updated_at = current_timestamp
    returning cfs_submission_id into v_event_cfs_submission_id;

    insert into group_cfs_submission_assignment (
        group_cfs_submission_id, event_id, event_cfs_submission_id, assigned_by
    ) values (
        p_group_cfs_submission_id, p_event_id, v_event_cfs_submission_id, p_reviewer_id
    )
    on conflict (group_cfs_submission_id, event_id) do update
        set event_cfs_submission_id = excluded.event_cfs_submission_id,
            assigned_by = excluded.assigned_by,
            assigned_at = current_timestamp;
    return v_event_cfs_submission_id;
end;
$$ language plpgsql;

create or replace function update_group_cfs_submission(
    p_reviewer_id uuid,
    p_group_id uuid,
    p_group_cfs_submission_id uuid,
    p_submission jsonb
)
returns void as $$
declare
    v_label_ids uuid[];
    v_rating_stars int;
begin
    if p_submission->>'status_id' not in (
        'approved', 'information-requested', 'not-reviewed', 'rejected'
    ) then
        raise exception 'invalid submission status';
    end if;
    if coalesce(jsonb_array_length(p_submission->'label_ids'), 0) > 10 then
        raise exception 'too many submission labels';
    end if;
    v_label_ids := array(
        select value::uuid from jsonb_array_elements_text(
            coalesce(p_submission->'label_ids', '[]'::jsonb)
        )
    );
    if exists (
        select 1 from unnest(v_label_ids) label_id where not exists (
            select 1 from group_cfs_label
            where group_id = p_group_id and group_cfs_label_id = label_id
        )
    ) then
        raise exception 'invalid rolling cfs labels';
    end if;
    if p_submission ? 'rating_stars' then
        v_rating_stars := (p_submission->>'rating_stars')::int;
        if v_rating_stars not between 0 and 5 then
            raise exception 'invalid rating stars';
        end if;
    end if;
    update group_cfs_submission set
        action_required_message = nullif(p_submission->>'action_required_message', ''),
        reviewed_by = p_reviewer_id,
        status_id = p_submission->>'status_id',
        updated_at = current_timestamp
    where group_cfs_submission_id = p_group_cfs_submission_id and group_id = p_group_id
    and status_id <> 'withdrawn';
    if not found then
        raise exception 'rolling cfs submission not found';
    end if;
    delete from group_cfs_submission_label where group_cfs_submission_id = p_group_cfs_submission_id;
    insert into group_cfs_submission_label (group_cfs_submission_id, group_cfs_label_id)
    select p_group_cfs_submission_id, label_id from unnest(v_label_ids) label_id;
    if p_submission ? 'rating_stars' then
        if v_rating_stars = 0 then
            delete from group_cfs_submission_rating
            where group_cfs_submission_id = p_group_cfs_submission_id and reviewer_id = p_reviewer_id;
        else
            insert into group_cfs_submission_rating (
                group_cfs_submission_id, reviewer_id, stars, comments
            ) values (
                p_group_cfs_submission_id, p_reviewer_id, v_rating_stars,
                nullif(p_submission->>'rating_comment', '')
            )
            on conflict (group_cfs_submission_id, reviewer_id) do update set
                stars = excluded.stars, comments = excluded.comments, updated_at = current_timestamp;
        end if;
    end if;
end;
$$ language plpgsql;

create or replace function get_group_cfs(
    p_alliance_id uuid,
    p_group_slug text
)
returns jsonb as $$
    select jsonb_build_object(
        'description', gc.description,
        'enabled', gc.enabled,
        'group_id', g.group_id,
        'group_name', g.name,
        'group_slug', coalesce(g.slug_pretty, g.slug),
        'labels', coalesce((
            select jsonb_agg(jsonb_build_object(
                'color', l.color, 'group_cfs_label_id', l.group_cfs_label_id, 'name', l.name
            ) order by l.name, l.group_cfs_label_id)
            from group_cfs_label l where l.group_id = g.group_id
        ), '[]'::jsonb)
    )
    from "group" g
    join group_cfs gc on gc.group_id = g.group_id and gc.enabled
    where g.alliance_id = p_alliance_id and g.active
    and (g.slug = p_group_slug or g.slug_pretty = p_group_slug)
$$ language sql stable;

create or replace function list_user_session_proposals_for_group_cfs(
    p_user_id uuid,
    p_group_id uuid
)
returns jsonb as $$
    select coalesce(jsonb_agg(jsonb_build_object(
        'is_submitted', exists (
            select 1 from group_cfs_submission gcs
            where gcs.group_id = p_group_id
            and gcs.session_proposal_id = sp.session_proposal_id
            and gcs.status_id <> 'withdrawn'
        ),
        'session_proposal_id', sp.session_proposal_id,
        'title', sp.title
    ) order by sp.title, sp.session_proposal_id), '[]'::jsonb)
    from session_proposal sp
    where sp.user_id = p_user_id
    and sp.session_proposal_status_id = 'ready-for-submission'
$$ language sql stable;

create or replace function get_group_cfs_dashboard(p_group_id uuid)
returns jsonb as $$
    select jsonb_build_object(
        'description', gc.description,
        'enabled', coalesce(gc.enabled, false),
        'labels', coalesce((
            select jsonb_agg(jsonb_build_object(
                'color', l.color, 'event_cfs_label_id', l.group_cfs_label_id, 'name', l.name
            ) order by l.name, l.group_cfs_label_id)
            from group_cfs_label l where l.group_id = p_group_id
        ), '[]'::jsonb)
    )
    from (select p_group_id as group_id) input
    left join group_cfs gc using (group_id)
$$ language sql stable;

create or replace function list_group_cfs_submissions(p_group_id uuid)
returns jsonb as $$
    select coalesce(jsonb_agg(jsonb_build_object(
        'assignments', coalesce((
            select jsonb_agg(jsonb_build_object(
                'assigned_at', extract(epoch from a.assigned_at)::bigint,
                'event_id', e.event_id,
                'event_name', e.name,
                'event_starts_at', extract(epoch from e.starts_at)::bigint
            ) order by a.assigned_at desc)
            from group_cfs_submission_assignment a
            join event e on e.event_id = a.event_id
            where a.group_cfs_submission_id = gcs.group_cfs_submission_id
        ), '[]'::jsonb),
        'average_rating', (
            select round(avg(r.stars)::numeric, 1)::double precision
            from group_cfs_submission_rating r
            where r.group_cfs_submission_id = gcs.group_cfs_submission_id
        ),
        'created_at', extract(epoch from gcs.created_at)::bigint,
        'group_cfs_submission_id', gcs.group_cfs_submission_id,
        'labels', coalesce((
            select jsonb_agg(jsonb_build_object(
                'color', l.color, 'group_cfs_label_id', l.group_cfs_label_id, 'name', l.name
            ) order by l.name, l.group_cfs_label_id)
            from group_cfs_submission_label sl
            join group_cfs_label l on l.group_cfs_label_id = sl.group_cfs_label_id
            where sl.group_cfs_submission_id = gcs.group_cfs_submission_id
        ), '[]'::jsonb),
        'ratings_count', (
            select count(*)::int from group_cfs_submission_rating r
            where r.group_cfs_submission_id = gcs.group_cfs_submission_id
        ),
        'speaker_name', coalesce(u.name, u.username),
        'status_id', gcs.status_id,
        'status_name', css.display_name,
        'title', sp.title
    ) order by gcs.created_at desc, gcs.group_cfs_submission_id), '[]'::jsonb)
    from group_cfs_submission gcs
    join session_proposal sp on sp.session_proposal_id = gcs.session_proposal_id
    join "user" u on u.user_id = sp.user_id
    join cfs_submission_status css on css.cfs_submission_status_id = gcs.status_id
    where gcs.group_id = p_group_id and gcs.status_id <> 'withdrawn'
$$ language sql stable;

create or replace function list_group_cfs_assignment_events(p_group_id uuid)
returns jsonb as $$
    select coalesce(jsonb_agg(jsonb_build_object(
        'event_id', e.event_id,
        'name', e.name,
        'starts_at', extract(epoch from e.starts_at)::bigint
    ) order by e.starts_at nulls last, e.name), '[]'::jsonb)
    from event e
    where e.group_id = p_group_id
    and e.deleted = false
    and e.canceled = false
$$ language sql stable;
