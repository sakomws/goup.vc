create or replace function opportunity_row_json(p opportunity_board_row)
returns jsonb language sql stable as $$
    select jsonb_strip_nulls(jsonb_build_object(
        'source_id', p.source_id,
        'source_kind', p.source_kind,
        'kind', p.kind,
        'title', p.title,
        'organization_name', p.organization_name,
        'summary', p.summary,
        'description', p.description,
        'apply_url', p.apply_url,
        'location', p.location,
        'remote', p.remote,
        'members_only', p.members_only,
        'tags', p.tags,
        'published', p.published,
        'created_at', extract(epoch from p.created_at),
        'closes_at', case when p.closes_at is null then null
            else extract(epoch from p.closes_at) end,
        'posted_by_user_id', p.posted_by_user_id,
        'poster_username', p.poster_username,
        'poster_name', p.poster_name
    ))
$$;

create or replace function opportunity_rows(p_include_members_only boolean default false)
returns setof opportunity_board_row language sql stable as $$
    select o.opportunity_id, 'native'::text, o.kind, o.title,
        o.organization_name, o.summary, o.description, o.apply_url,
        o.location, o.remote, o.members_only, o.tags, o.published,
        o.created_at, o.closes_at, o.posted_by_user_id, u.username, u.name
    from opportunity o
    join "user" u on u.user_id = o.posted_by_user_id
    where o.published
      and (o.closes_at is null or o.closes_at > current_timestamp)
      and (p_include_members_only or not o.members_only)

    union all

    select j.job_id, 'job', 'job', j.title, j.company_name, j.summary,
        j.description, j.apply_url, j.location, j.remote, j.members_only,
        j.tags, j.published, j.created_at, j.expires_at,
        j.posted_by_user_id, u.username, u.name
    from jobs_job j
    join "user" u on u.user_id = j.posted_by_user_id
    where j.published and j.expires_at > current_timestamp
      and (p_include_members_only or not j.members_only)

    union all

    select e.event_id, 'event-cfs', 'cfs',
        'Speak at ' || e.name, g.name,
        coalesce(e.cfs_description, e.description_short, 'Submit a talk proposal.'),
        coalesce(e.cfs_description, e.description),
        format('/%s/group/%s/event/%s', a.name, coalesce(g.slug_pretty, g.slug), e.slug),
        nullif(concat_ws(', ', e.venue_city, e.venue_country_name), ''),
        e.event_kind_id in ('virtual', 'hybrid'), false, coalesce(e.tags, '{}'),
        true, coalesce(e.published_at, e.created_at), e.cfs_ends_at,
        null::uuid, null::text, null::text
    from event e
    join "group" g using (group_id)
    join alliance a using (alliance_id)
    where e.published and not e.deleted and not e.canceled and e.cfs_enabled
      and coalesce(e.cfs_starts_at, '-infinity'::timestamptz) <= current_timestamp
      and coalesce(e.cfs_ends_at, 'infinity'::timestamptz) > current_timestamp
      and g.active and not g.deleted and a.active

    union all

    select g.group_id, 'group-cfs', 'cfs',
        'Speak with ' || g.name, g.name,
        coalesce(gc.description, 'Submit a reusable talk proposal.'),
        coalesce(gc.description, 'Submit a reusable talk proposal to this community.'),
        format('/%s/group/%s/cfs', a.name, coalesce(g.slug_pretty, g.slug)),
        null::text, true, false,
        coalesce((select array_agg(l.name order by l.name)
            from group_cfs_label l where l.group_id = g.group_id), '{}'),
        true, gc.created_at, null::timestamptz,
        null::uuid, null::text, null::text
    from group_cfs gc
    join "group" g using (group_id)
    join alliance a using (alliance_id)
    where gc.enabled and g.active and not g.deleted and a.active
$$;

create or replace function search_opportunities(p_filters jsonb)
returns jsonb language plpgsql stable as $$
declare
    v_limit int := least(greatest(coalesce((p_filters->>'limit')::int, 20), 1), 100);
    v_offset int := greatest(coalesce((p_filters->>'offset')::int, 0), 0);
    v_query text := nullif(btrim(p_filters->>'query'), '');
    v_kind text := nullif(btrim(p_filters->>'kind'), '');
    v_location text := nullif(btrim(p_filters->>'location'), '');
    v_remote boolean := (p_filters->>'remote')::boolean;
    v_include_members boolean :=
        coalesce((p_filters->>'include_members_only')::boolean, false);
begin
    return (
        with matches as (
            select r.*
            from opportunity_rows(v_include_members) r
            where (v_kind is null or r.kind = v_kind)
              and (v_remote is null or r.remote = v_remote)
              and (v_location is null or r.location ilike
                    '%' || escape_ilike_pattern(v_location) || '%' escape '\')
              and (
                v_query is null
                or r.title ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
                or r.organization_name ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
                or r.summary ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
                or exists (select 1 from unnest(r.tags) tag where tag ilike
                    '%' || escape_ilike_pattern(v_query) || '%' escape '\')
              )
        ),
        paged as (
            select * from matches
            order by created_at desc, source_kind, source_id
            limit v_limit offset v_offset
        )
        select jsonb_build_object(
            'opportunities', coalesce((
                select jsonb_agg(opportunity_row_json(paged)
                    order by created_at desc, source_kind, source_id) from paged
            ), '[]'::jsonb),
            'total', (select count(*) from matches)
        )
    );
end;
$$;

create or replace function get_opportunity(
    p_source_kind text,
    p_source_id uuid,
    p_include_members_only boolean default false
) returns jsonb language sql stable as $$
    select opportunity_row_json(r)
    from opportunity_rows(p_include_members_only) r
    where r.source_kind = p_source_kind and r.source_id = p_source_id
$$;

create or replace function list_user_opportunities(p_user_id uuid, p_filters jsonb)
returns jsonb language sql stable as $$
    with matches as (
        select o.*, u.username as poster_username, u.name as poster_name
        from opportunity o
        join "user" u on u.user_id = o.posted_by_user_id
        where o.posted_by_user_id = p_user_id
    ), paged as (
        select * from matches order by created_at desc
        limit least(coalesce((p_filters->>'limit')::int, 50), 100)
        offset greatest(coalesce((p_filters->>'offset')::int, 0), 0)
    )
    select jsonb_build_object(
        'opportunities', coalesce((
            select jsonb_agg(jsonb_strip_nulls(jsonb_build_object(
                'source_id', opportunity_id, 'source_kind', 'native',
                'kind', kind, 'title', title,
                'organization_name', organization_name, 'summary', summary,
                'description', description, 'apply_url', apply_url,
                'location', location, 'remote', remote,
                'members_only', members_only, 'tags', tags,
                'published', published, 'created_at', extract(epoch from created_at),
                'closes_at', case when closes_at is null then null
                    else extract(epoch from closes_at) end,
                'posted_by_user_id', posted_by_user_id,
                'poster_username', poster_username, 'poster_name', poster_name
            )) order by created_at desc) from paged
        ), '[]'::jsonb),
        'total', (select count(*) from matches)
    )
$$;

create or replace function add_opportunity(
    p_user_id uuid, p_input jsonb, p_tags text[]
) returns uuid language plpgsql as $$
declare v_id uuid; v_slug text;
begin
    v_slug := generate_slug();
    insert into opportunity (
        posted_by_user_id, kind, title, slug, organization_name, summary,
        description, apply_url, location, remote, members_only, tags,
        opens_at, closes_at
    ) values (
        p_user_id, p_input->>'kind', btrim(p_input->>'title'), v_slug,
        btrim(p_input->>'organization_name'), btrim(p_input->>'summary'),
        btrim(p_input->>'description'), btrim(p_input->>'apply_url'),
        nullif(btrim(p_input->>'location'), ''),
        coalesce((p_input->>'remote')::boolean, false),
        coalesce((p_input->>'members_only')::boolean, false),
        coalesce(p_tags, '{}'),
        nullif(p_input->>'opens_at', '')::timestamptz,
        nullif(p_input->>'closes_at', '')::timestamptz
    ) returning opportunity_id into v_id;
    return v_id;
end;
$$;

create or replace function update_opportunity(
    p_user_id uuid, p_opportunity_id uuid, p_input jsonb, p_tags text[]
) returns void language plpgsql as $$
begin
    update opportunity set
        kind = p_input->>'kind', title = btrim(p_input->>'title'),
        organization_name = btrim(p_input->>'organization_name'),
        summary = btrim(p_input->>'summary'),
        description = btrim(p_input->>'description'),
        apply_url = btrim(p_input->>'apply_url'),
        location = nullif(btrim(p_input->>'location'), ''),
        remote = coalesce((p_input->>'remote')::boolean, false),
        members_only = coalesce((p_input->>'members_only')::boolean, false),
        tags = coalesce(p_tags, '{}'),
        opens_at = nullif(p_input->>'opens_at', '')::timestamptz,
        closes_at = nullif(p_input->>'closes_at', '')::timestamptz,
        updated_at = current_timestamp
    where opportunity_id = p_opportunity_id and posted_by_user_id = p_user_id;
    if not found then raise exception 'opportunity not found'; end if;
end;
$$;

create or replace function delete_opportunity(p_user_id uuid, p_opportunity_id uuid)
returns void language plpgsql as $$
begin
    delete from opportunity
    where opportunity_id = p_opportunity_id and posted_by_user_id = p_user_id;
    if not found then raise exception 'opportunity not found'; end if;
end;
$$;

create or replace function update_opportunity_published(
    p_user_id uuid, p_opportunity_id uuid, p_published boolean
) returns void language plpgsql as $$
begin
    update opportunity set published = p_published, updated_at = current_timestamp
    where opportunity_id = p_opportunity_id and posted_by_user_id = p_user_id;
    if not found then raise exception 'opportunity not found'; end if;
end;
$$;

create or replace function upsert_opportunity_saved_search(
    p_user_id uuid, p_saved_search_id uuid, p_input jsonb
) returns uuid language plpgsql as $$
declare v_id uuid := coalesce(p_saved_search_id, gen_random_uuid());
begin
    insert into opportunity_saved_search (
        opportunity_saved_search_id, user_id, name, filters, frequency, active, next_run_at
    ) values (
        v_id, p_user_id, btrim(p_input->>'name'),
        coalesce(p_input->'filters', '{}'), p_input->>'frequency', false, null
    )
    on conflict (opportunity_saved_search_id) do update set
        name = excluded.name, filters = excluded.filters,
        frequency = excluded.frequency, active = false, next_run_at = null,
        updated_at = current_timestamp
    where opportunity_saved_search.user_id = p_user_id;
    if not found then raise exception 'saved search not found'; end if;
    return v_id;
end;
$$;

create or replace function list_opportunity_saved_searches(p_user_id uuid)
returns jsonb language sql stable as $$
    select coalesce(jsonb_agg(jsonb_build_object(
        'opportunity_saved_search_id', opportunity_saved_search_id,
        'name', name, 'filters', filters, 'frequency', frequency,
        'active', active,
        'next_run_at', case when next_run_at is null then null
            else extract(epoch from next_run_at) end
    ) order by created_at desc), '[]'::jsonb)
    from opportunity_saved_search where user_id = p_user_id
$$;

create or replace function activate_opportunity_saved_search(
    p_user_id uuid, p_saved_search_id uuid
) returns void language plpgsql as $$
begin
    update opportunity_saved_search set active = true,
        next_run_at = case frequency when 'daily' then current_timestamp + interval '1 day'
            else current_timestamp + interval '7 days' end,
        updated_at = current_timestamp
    where opportunity_saved_search_id = p_saved_search_id and user_id = p_user_id;
    if not found then raise exception 'saved search not found'; end if;
end;
$$;

create or replace function delete_opportunity_saved_search(
    p_user_id uuid, p_saved_search_id uuid
) returns void language plpgsql as $$
begin
    delete from opportunity_saved_search
    where opportunity_saved_search_id = p_saved_search_id and user_id = p_user_id;
    if not found then raise exception 'saved search not found'; end if;
end;
$$;
