create or replace function add_mock_interview_feedback(
    p_reviewer_user_id uuid,
    p_session_id uuid,
    p_input jsonb
)
returns jsonb language plpgsql as $$
declare
    v_session mock_interview_session;
    v_request mock_interview_request;
    v_reviewer_role text;
    v_reviewee_user_id uuid;
    v_feedback mock_interview_feedback;
    v_good_reviews int;
    v_avg_score numeric;
begin
    select * into v_session
    from mock_interview_session s
    where s.mock_interview_session_id = p_session_id
    for update;

    if v_session.mock_interview_session_id is null then
        raise exception 'session not found';
    end if;

    select * into v_request
    from mock_interview_request r
    where r.mock_interview_request_id = v_session.mock_interview_request_id;

    if v_request.interviewer_user_id = p_reviewer_user_id then
        v_reviewer_role := 'interviewer';
        v_reviewee_user_id := v_request.interviewee_user_id;
    elsif v_request.interviewee_user_id = p_reviewer_user_id then
        v_reviewer_role := 'interviewee';
        v_reviewee_user_id := v_request.interviewer_user_id;
    else
        raise exception 'not a session participant';
    end if;

    insert into mock_interview_feedback (
        mock_interview_session_id,
        reviewer_user_id,
        reviewee_user_id,
        reviewer_role,
        communication,
        technical_depth,
        problem_solving,
        role_readiness,
        helpfulness,
        feedback_quality,
        would_recommend,
        suggested_next_steps
    ) values (
        p_session_id,
        p_reviewer_user_id,
        v_reviewee_user_id,
        v_reviewer_role,
        (p_input->>'communication')::smallint,
        (p_input->>'technical_depth')::smallint,
        (p_input->>'problem_solving')::smallint,
        (p_input->>'role_readiness')::smallint,
        (p_input->>'helpfulness')::smallint,
        (p_input->>'feedback_quality')::smallint,
        (p_input->>'would_recommend')::boolean,
        nullif(trim(p_input->>'suggested_next_steps'), '')
    )
    on conflict (mock_interview_session_id, reviewer_user_id) do update set
        communication = excluded.communication,
        technical_depth = excluded.technical_depth,
        problem_solving = excluded.problem_solving,
        role_readiness = excluded.role_readiness,
        helpfulness = excluded.helpfulness,
        feedback_quality = excluded.feedback_quality,
        would_recommend = excluded.would_recommend,
        suggested_next_steps = excluded.suggested_next_steps
    returning * into v_feedback;

    if not exists (
        select 1
        from mock_interview_feedback f
        where f.mock_interview_session_id = p_session_id
        and f.reviewer_role = 'interviewer'
    ) or not exists (
        select 1
        from mock_interview_feedback f
        where f.mock_interview_session_id = p_session_id
        and f.reviewer_role = 'interviewee'
    ) then
        return jsonb_build_object('feedback_id', v_feedback.mock_interview_feedback_id);
    end if;

    update mock_interview_session
    set status = 'completed',
        completed_at = current_timestamp
    where mock_interview_session_id = p_session_id;

    update mock_interview_request
    set status = 'completed'
    where mock_interview_request_id = v_request.mock_interview_request_id;

    update mock_interview_profile
    set completed_sessions = completed_sessions + 1,
        updated_at = current_timestamp
    where user_id in (v_request.interviewee_user_id, v_request.interviewer_user_id);

    select count(*)::int,
        avg(
            coalesce(f.helpfulness, f.feedback_quality, f.communication)::numeric
        )
    into v_good_reviews, v_avg_score
    from mock_interview_feedback f
    where f.reviewee_user_id = v_request.interviewer_user_id
    and f.reviewer_role = 'interviewee'
    and coalesce(f.would_recommend, false) = true;

    update mock_interview_profile
    set reputation_score = coalesce(v_avg_score, reputation_score),
        interviewer_badge = (v_good_reviews >= 3 and coalesce(v_avg_score, 0) >= 4),
        updated_at = current_timestamp
    where user_id = v_request.interviewer_user_id;

    return jsonb_build_object(
        'feedback_id', v_feedback.mock_interview_feedback_id,
        'session_completed', true
    );
end;
$$;
