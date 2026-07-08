create or replace function mock_interview_seniority_rank(p_seniority text)
returns int language sql immutable as $$
    select case p_seniority
        when 'junior' then 1
        when 'mid' then 2
        when 'senior' then 3
        when 'staff_plus' then 4
        else 0
    end;
$$;

create or replace function mock_interview_profile_json(p_profile mock_interview_profile)
returns jsonb language sql stable as $$
    select jsonb_build_object(
        'user_id', p_profile.user_id,
        'role_intent', p_profile.role_intent,
        'timezone_region', p_profile.timezone_region,
        'seniority', p_profile.seniority,
        'interview_types', p_profile.interview_types,
        'target_company_types', p_profile.target_company_types,
        'availability_slots', p_profile.availability_slots,
        'linkedin_url', p_profile.linkedin_url,
        'github_url', p_profile.github_url,
        'resume_url', p_profile.resume_url,
        'enabled', p_profile.enabled,
        'reputation_score', p_profile.reputation_score,
        'completed_sessions', p_profile.completed_sessions,
        'interviewer_badge', p_profile.interviewer_badge,
        'username', u.username,
        'name', u.name,
        'photo_url', u.photo_url,
        'title', u.title,
        'company', u.company,
        'created_at', extract(epoch from p_profile.created_at)::bigint,
        'updated_at', case
            when p_profile.updated_at is null then null
            else extract(epoch from p_profile.updated_at)::bigint
        end
    )
    from "user" u
    where u.user_id = p_profile.user_id;
$$;
