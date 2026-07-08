create or replace function mock_interview_request_json(p_request mock_interview_request)
returns jsonb language sql stable as $$
    select jsonb_build_object(
        'mock_interview_request_id', p_request.mock_interview_request_id,
        'interviewee_user_id', p_request.interviewee_user_id,
        'interviewer_user_id', p_request.interviewer_user_id,
        'interview_type', p_request.interview_type,
        'message', p_request.message,
        'status', p_request.status,
        'created_at', extract(epoch from p_request.created_at)::bigint,
        'responded_at', case
            when p_request.responded_at is null then null
            else extract(epoch from p_request.responded_at)::bigint
        end,
        'interviewee_username', ie.username,
        'interviewee_name', ie.name,
        'interviewee_photo_url', ie.photo_url,
        'interviewer_username', ir.username,
        'interviewer_name', ir.name,
        'interviewer_photo_url', ir.photo_url,
        'mock_interview_session_id', s.mock_interview_session_id
    )
    from "user" ie
    join "user" ir on ir.user_id = p_request.interviewer_user_id
    left join mock_interview_session s
        on s.mock_interview_request_id = p_request.mock_interview_request_id
    where ie.user_id = p_request.interviewee_user_id;
$$;
