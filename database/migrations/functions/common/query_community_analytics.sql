-- Returns aggregate-only community analytics for public, completed event activity.
-- No user, attendee, event, or group identifiers are exposed.
create or replace function query_community_analytics(
    p_start_at timestamp with time zone,
    p_end_at timestamp with time zone,
    p_country_codes text[] default null,
    p_alliance_name text default null,
    p_limit integer default 10
) returns jsonb
language plpgsql
as $$
begin
    if p_start_at is null or p_end_at is null or p_start_at >= p_end_at then
        raise exception 'start_at must be before end_at';
    end if;

    if p_limit is null or p_limit < 1 or p_limit > 50 then
        raise exception 'limit must be between 1 and 50';
    end if;

    return (
        with filtered_events as (
            select
                e.event_id,
                g.name as community_name,
                upper(coalesce(nullif(e.venue_country_code, ''), nullif(g.country_code, ''))) as country_code
            from event e
            join "group" g on g.group_id = e.group_id
            join alliance a on a.alliance_id = g.alliance_id
            where e.starts_at >= p_start_at
              and e.starts_at < p_end_at
              and e.published = true
              and e.canceled = false
              and e.deleted = false
              and e.test_event = false
              and g.active = true
              and g.deleted = false
              and a.active = true
              and (p_alliance_name is null or a.name = p_alliance_name)
              and (
                  coalesce(cardinality(p_country_codes), 0) = 0
                  or upper(coalesce(nullif(e.venue_country_code, ''), nullif(g.country_code, '')))
                     = any (
                         select upper(country_code)
                         from unnest(p_country_codes) as country_code
                     )
              )
        ),
        event_metrics as (
            select
                fe.event_id,
                fe.community_name,
                fe.country_code,
                count(ea.user_id) filter (where ea.status = 'confirmed')::integer as attendee_count
            from filtered_events fe
            left join event_attendee ea on ea.event_id = fe.event_id
            group by fe.event_id, fe.community_name, fe.country_code
        ),
        totals as (
            select
                count(*)::integer as event_count,
                coalesce(sum(attendee_count), 0)::integer as attendee_count
            from event_metrics
        ),
        by_country as (
            select coalesce(
                jsonb_agg(
                    jsonb_build_object(
                        'country_code', country_code,
                        'event_count', event_count,
                        'attendee_count', attendee_count
                    )
                    order by attendee_count desc, event_count desc, country_code
                ),
                '[]'::jsonb
            ) as value
            from (
                select
                    coalesce(nullif(country_code, ''), 'UNKNOWN') as country_code,
                    count(*)::integer as event_count,
                    coalesce(sum(attendee_count), 0)::integer as attendee_count
                from event_metrics
                group by coalesce(nullif(country_code, ''), 'UNKNOWN')
                order by attendee_count desc, event_count desc, country_code
                limit p_limit
            ) rows
        ),
        top_communities as (
            select coalesce(
                jsonb_agg(
                    jsonb_build_object(
                        'community_name', community_name,
                        'event_count', event_count,
                        'attendee_count', attendee_count
                    )
                    order by attendee_count desc, event_count desc, community_name
                ),
                '[]'::jsonb
            ) as value
            from (
                select
                    community_name,
                    count(*)::integer as event_count,
                    coalesce(sum(attendee_count), 0)::integer as attendee_count
                from event_metrics
                group by community_name
                order by attendee_count desc, event_count desc, community_name
                limit p_limit
            ) rows
        )
        select jsonb_build_object(
            'period', jsonb_build_object('start_at', p_start_at, 'end_at', p_end_at),
            'event_count', totals.event_count,
            'attendee_count', totals.attendee_count,
            'by_country', by_country.value,
            'top_communities', top_communities.value
        )
        from totals
        cross join by_country
        cross join top_communities
    );
end;
$$;
