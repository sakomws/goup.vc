-- Enqueue due saved-search digests. The digest-run primary key makes this safe
-- across retries, and enqueue_notification applies the user's optional-email
-- preference before creating delivery rows.
create or replace function enqueue_due_opportunity_digests(p_base_url text)
returns int language plpgsql as $$
declare
    v_due record;
    v_scheduled_for timestamptz;
    v_matches jsonb;
    v_match_count int;
    v_inserted int;
    v_enqueued int := 0;
begin
    if not pg_try_advisory_xact_lock(hashtextextended('ocg:opportunity-digests', 0)) then
        return 0;
    end if;

    for v_due in
        select s.*, u.email_verified, u.optional_notifications_enabled,
            (select site.theme from site order by site.created_at desc limit 1) as theme
        from opportunity_saved_search s
        join "user" u using (user_id)
        where s.active and s.next_run_at <= current_timestamp
        order by s.next_run_at, s.opportunity_saved_search_id
        for update of s skip locked
    loop
        v_scheduled_for := v_due.next_run_at;

        select coalesce(jsonb_agg(opportunity_row_json(r)
                    order by r.created_at desc), '[]'::jsonb)
        into v_matches
        from (
            select row_source.*
            from opportunity_rows(true) row_source
            where row_source.created_at > coalesce(v_due.last_run_at, v_due.created_at)
              and (nullif(v_due.filters->>'kind', '') is null
                   or row_source.kind = v_due.filters->>'kind')
              and ((v_due.filters->>'remote') is null
                   or row_source.remote = (v_due.filters->>'remote')::boolean)
              and (nullif(v_due.filters->>'location', '') is null
                   or row_source.location ilike '%' ||
                       escape_ilike_pattern(v_due.filters->>'location') || '%' escape '\')
              and (
                nullif(v_due.filters->>'query', '') is null
                or row_source.title ilike '%' ||
                    escape_ilike_pattern(v_due.filters->>'query') || '%' escape '\'
                or row_source.organization_name ilike '%' ||
                    escape_ilike_pattern(v_due.filters->>'query') || '%' escape '\'
                or row_source.summary ilike '%' ||
                    escape_ilike_pattern(v_due.filters->>'query') || '%' escape '\'
              )
            limit 20
        ) r;
        v_match_count := jsonb_array_length(v_matches);

        insert into opportunity_digest_run (
            opportunity_saved_search_id, scheduled_for, match_count
        ) values (
            v_due.opportunity_saved_search_id, v_scheduled_for, v_match_count
        ) on conflict do nothing;
        get diagnostics v_inserted = row_count;

        if v_inserted = 1 and v_match_count > 0
           and v_due.email_verified and v_due.optional_notifications_enabled then
            perform enqueue_notification(
                'opportunity-digest',
                jsonb_build_object(
                    'search_name', v_due.name,
                    'frequency', v_due.frequency,
                    'match_count', v_match_count,
                    'opportunities', v_matches,
                    'board_link', regexp_replace(coalesce(p_base_url, ''), '/+$', ''),
                    'manage_link', regexp_replace(coalesce(p_base_url, ''), '/+$', '')
                        || '/dashboard/opportunities',
                    'theme', v_due.theme
                ),
                '[]'::jsonb,
                array[v_due.user_id]
            );
            update opportunity_digest_run set enqueued_at = current_timestamp
            where opportunity_saved_search_id = v_due.opportunity_saved_search_id
              and scheduled_for = v_scheduled_for;
            v_enqueued := v_enqueued + 1;
        end if;

        update opportunity_saved_search set
            last_run_at = current_timestamp,
            next_run_at = current_timestamp + case v_due.frequency
                when 'daily' then interval '1 day' else interval '7 days' end,
            updated_at = current_timestamp
        where opportunity_saved_search_id = v_due.opportunity_saved_search_id;
    end loop;

    return v_enqueued;
end;
$$;
