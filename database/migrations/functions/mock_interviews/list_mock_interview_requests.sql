create or replace function list_mock_interview_requests(p_user_id uuid)
returns jsonb language sql stable as $$
    select coalesce(
        jsonb_agg(
            mock_interview_request_json(r)
            order by r.created_at desc
        ),
        '[]'::jsonb
    )
    from mock_interview_request r
    where r.interviewee_user_id = p_user_id
    or r.interviewer_user_id = p_user_id;
$$;
