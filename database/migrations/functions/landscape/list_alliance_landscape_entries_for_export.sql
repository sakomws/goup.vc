create or replace function list_alliance_landscape_entries_for_export(
    p_alliance_id uuid,
    p_filters jsonb
) returns jsonb language plpgsql stable as $$
declare
    v_query text := nullif(trim(p_filters->>'query'), '');
    v_entries jsonb;
begin
    with matches as (
        select le.*
        from landscape_entry le
        left join landscape_accelerator_profile lap on lap.landscape_entry_id = le.landscape_entry_id
        where le.alliance_id = p_alliance_id
        and (
            v_query is null
            or le.name ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or le.summary ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or le.description ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or le.category ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or lap.application_url ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or lap.curriculum_url ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or lap.cohort_status ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or lap.weekly_agenda::text ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            or exists (
                select 1 from unnest(le.tags) tag
                where tag ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            )
            or exists (
                select 1 from unnest(lap.tracks) track
                where track ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
            )
        )
    )
    select coalesce(
        jsonb_agg(landscape_entry_json(matches) order by matches.created_at desc, matches.landscape_entry_id desc),
        '[]'::jsonb
    )
    into v_entries
    from matches;

    return jsonb_build_object(
        'entries', v_entries,
        'total', jsonb_array_length(v_entries)
    );
end;
$$;
