create or replace function get_mock_interview_profile(p_user_id uuid)
returns jsonb language sql stable as $$
    select mock_interview_profile_json(p)
    from mock_interview_profile p
    where p.user_id = p_user_id;
$$;
