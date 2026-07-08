create or replace function search_mock_interview_matches(
    p_user_id uuid,
    p_filters jsonb default '{}'::jsonb
)
returns jsonb language plpgsql stable as $$
declare
    v_limit int := least(coalesce((p_filters->>'limit')::int, 5), 10);
    v_interview_type text := nullif(trim(p_filters->>'interview_type'), '');
    v_viewer mock_interview_profile;
    v_matches jsonb;
begin
    select * into v_viewer
    from mock_interview_profile
    where user_id = p_user_id
    and enabled = true;

    if v_viewer.user_id is null then
        return jsonb_build_object('matches', '[]'::jsonb, 'total', 0);
    end if;

    with candidates as (
        select
            p.*,
            u.username,
            (
                case when p.timezone_region = v_viewer.timezone_region then 30 else 0 end
                + case
                    when v_interview_type is not null
                        and v_interview_type = any (p.interview_types) then 25
                    when v_interview_type is null
                        and p.interview_types && v_viewer.interview_types then 20
                    else 0
                  end
                + case
                    when p.target_company_types && v_viewer.target_company_types then 15
                    else 0
                  end
                + case
                    when mock_interview_seniority_rank(p.seniority)
                        >= mock_interview_seniority_rank(v_viewer.seniority) then 20
                    else 0
                  end
                + least(coalesce(p.reputation_score, 0), 5)::int * 2
                + case when p.interviewer_badge then 5 else 0 end
            )::int as match_score
        from mock_interview_profile p
        join "user" u on u.user_id = p.user_id
        where p.enabled = true
        and p.user_id <> p_user_id
        and p.role_intent in ('interviewer', 'both')
        and mock_interview_seniority_rank(p.seniority)
            >= mock_interview_seniority_rank(v_viewer.seniority)
        and (
            v_interview_type is null
            or v_interview_type = any (p.interview_types)
        )
        and not exists (
            select 1
            from mock_interview_request r
            where r.interviewee_user_id = p_user_id
            and r.interviewer_user_id = p.user_id
            and r.status in ('pending', 'accepted')
        )
    ),
    ranked as (
        select *
        from candidates
        where match_score > 0
        order by match_score desc, reputation_score desc, completed_sessions desc
        limit v_limit
    )
    select coalesce(
        jsonb_agg(
            mock_interview_profile_json(ranked)
            || jsonb_build_object('match_score', ranked.match_score)
            order by ranked.match_score desc
        ),
        '[]'::jsonb
    )
    into v_matches
    from ranked;

    return jsonb_build_object(
        'matches', coalesce(v_matches, '[]'::jsonb),
        'total', coalesce(jsonb_array_length(v_matches), 0)
    );
end;
$$;
