create or replace function mock_interview_session_json(p_session mock_interview_session)
returns jsonb language sql stable as $$
    select jsonb_build_object(
        'mock_interview_session_id', p_session.mock_interview_session_id,
        'mock_interview_request_id', p_session.mock_interview_request_id,
        'meeting_url', p_session.meeting_url,
        'scheduled_at', case
            when p_session.scheduled_at is null then null
            else extract(epoch from p_session.scheduled_at)::bigint
        end,
        'status', p_session.status,
        'created_at', extract(epoch from p_session.created_at)::bigint,
        'completed_at', case
            when p_session.completed_at is null then null
            else extract(epoch from p_session.completed_at)::bigint
        end,
        'request', mock_interview_request_json(r)
    )
    from mock_interview_request r
    where r.mock_interview_request_id = p_session.mock_interview_request_id;
$$;

create or replace function get_mock_interview_session(
    p_user_id uuid,
    p_session_id uuid
)
returns jsonb language plpgsql stable as $$
declare
    v_session mock_interview_session;
    v_feedback jsonb;
begin
    select s.* into v_session
    from mock_interview_session s
    join mock_interview_request r on r.mock_interview_request_id = s.mock_interview_request_id
    where s.mock_interview_session_id = p_session_id
    and (r.interviewee_user_id = p_user_id or r.interviewer_user_id = p_user_id);

    if v_session.mock_interview_session_id is null then
        return null;
    end if;

    select coalesce(
        jsonb_agg(
            jsonb_build_object(
                'mock_interview_feedback_id', f.mock_interview_feedback_id,
                'reviewer_user_id', f.reviewer_user_id,
                'reviewee_user_id', f.reviewee_user_id,
                'reviewer_role', f.reviewer_role,
                'communication', f.communication,
                'technical_depth', f.technical_depth,
                'problem_solving', f.problem_solving,
                'role_readiness', f.role_readiness,
                'helpfulness', f.helpfulness,
                'feedback_quality', f.feedback_quality,
                'would_recommend', f.would_recommend,
                'suggested_next_steps', f.suggested_next_steps,
                'created_at', extract(epoch from f.created_at)::bigint
            )
            order by f.created_at asc
        ),
        '[]'::jsonb
    )
    into v_feedback
    from mock_interview_feedback f
    where f.mock_interview_session_id = p_session_id;

    return mock_interview_session_json(v_session)
        || jsonb_build_object('feedback', v_feedback);
end;
$$;
