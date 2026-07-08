create or replace function upsert_mock_interview_profile(
    p_user_id uuid,
    p_input jsonb
)
returns jsonb language plpgsql as $$
declare
    v_profile mock_interview_profile;
    v_interview_types text[];
    v_target_company_types text[];
begin
    v_interview_types := coalesce(
        array(
            select jsonb_array_elements_text(coalesce(p_input->'interview_types', '[]'::jsonb))
        ),
        '{}'::text[]
    );
    v_target_company_types := coalesce(
        array(
            select jsonb_array_elements_text(coalesce(p_input->'target_company_types', '[]'::jsonb))
        ),
        '{}'::text[]
    );

    insert into mock_interview_profile (
        user_id,
        role_intent,
        timezone_region,
        seniority,
        interview_types,
        target_company_types,
        availability_slots,
        linkedin_url,
        github_url,
        resume_url,
        enabled,
        updated_at
    ) values (
        p_user_id,
        p_input->>'role_intent',
        p_input->>'timezone_region',
        p_input->>'seniority',
        v_interview_types,
        v_target_company_types,
        coalesce(p_input->'availability_slots', '[]'::jsonb),
        nullif(trim(p_input->>'linkedin_url'), ''),
        nullif(trim(p_input->>'github_url'), ''),
        nullif(trim(p_input->>'resume_url'), ''),
        coalesce((p_input->>'enabled')::boolean, true),
        current_timestamp
    )
    on conflict (user_id) do update set
        role_intent = excluded.role_intent,
        timezone_region = excluded.timezone_region,
        seniority = excluded.seniority,
        interview_types = excluded.interview_types,
        target_company_types = excluded.target_company_types,
        availability_slots = excluded.availability_slots,
        linkedin_url = excluded.linkedin_url,
        github_url = excluded.github_url,
        resume_url = excluded.resume_url,
        enabled = excluded.enabled,
        updated_at = current_timestamp
    returning * into v_profile;

    return mock_interview_profile_json(v_profile);
end;
$$;
