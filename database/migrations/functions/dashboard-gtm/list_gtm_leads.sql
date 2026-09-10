create or replace function list_gtm_leads(
    p_alliance_id uuid,
    p_filters jsonb
) returns jsonb language plpgsql stable as $$
declare
    v_limit int := coalesce((p_filters->>'limit')::int, 50);
    v_offset int := coalesce((p_filters->>'offset')::int, 0);
    v_query text := nullif(trim(p_filters->>'query'), '');
    v_kind text := nullif(trim(p_filters->>'kind'), '');
    v_stage text := nullif(trim(p_filters->>'stage'), '');
    v_group_id uuid := nullif(p_filters->>'group_id', '')::uuid;
    v_owner_user_id uuid := nullif(p_filters->>'owner_user_id', '')::uuid;
    v_total int;
    v_leads jsonb;
    v_counts jsonb;
begin
    with matches as (
        select gl.*
        from gtm_lead gl
        where gl.alliance_id = p_alliance_id
          and (v_group_id is null or gl.group_id = v_group_id)
          and (v_kind is null or gl.kind = v_kind)
          and (v_stage is null or gl.stage = v_stage)
          and (v_owner_user_id is null or gl.owner_user_id = v_owner_user_id)
          and (
              v_query is null
              or gl.name ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
              or coalesce(gl.org_name, '') ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
              or coalesce(gl.email, '') ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
              or coalesce(gl.notes, '') ilike '%' || escape_ilike_pattern(v_query) || '%' escape '\'
          )
    ),
    counted as (
        select count(*)::int as total from matches
    ),
    paged as (
        select *
        from matches
        order by updated_at desc, gtm_lead_id desc
        limit v_limit
        offset v_offset
    ),
    stages as (
        select stage, count(*)::int as count
        from gtm_lead
        where alliance_id = p_alliance_id
          and (v_group_id is null or group_id = v_group_id)
        group by stage
    )
    select
        counted.total,
        coalesce(
            jsonb_agg(gtm_lead_json(paged) order by paged.updated_at desc, paged.gtm_lead_id desc)
                filter (where paged.gtm_lead_id is not null),
            '[]'::jsonb
        ),
        coalesce(
            (select jsonb_object_agg(stage, count) from stages),
            '{}'::jsonb
        )
    into v_total, v_leads, v_counts
    from counted
    left join paged on true
    group by counted.total;

    return jsonb_build_object(
        'leads', coalesce(v_leads, '[]'::jsonb),
        'total', coalesce(v_total, 0),
        'stage_counts', coalesce(v_counts, '{}'::jsonb)
    );
end;
$$;
