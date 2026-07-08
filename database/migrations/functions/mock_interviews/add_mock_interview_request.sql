create or replace function add_mock_interview_request(
    p_interviewee_user_id uuid,
    p_input jsonb
)
returns jsonb language plpgsql as $$
declare
    v_request mock_interview_request;
    v_interviewer_user_id uuid := (p_input->>'interviewer_user_id')::uuid;
    v_interview_type text := nullif(trim(p_input->>'interview_type'), '');
begin
    if v_interviewer_user_id is null then
        raise exception 'interviewer_user_id is required';
    end if;

    if v_interview_type is null then
        raise exception 'interview_type is required';
    end if;

    if v_interviewer_user_id = p_interviewee_user_id then
        raise exception 'cannot request yourself';
    end if;

    if not exists (
        select 1
        from mock_interview_profile p
        where p.user_id = p_interviewee_user_id
        and p.enabled = true
        and p.role_intent in ('interviewee', 'both')
    ) then
        raise exception 'interviewee profile required';
    end if;

    if not exists (
        select 1
        from mock_interview_profile p
        where p.user_id = v_interviewer_user_id
        and p.enabled = true
        and p.role_intent in ('interviewer', 'both')
        and v_interview_type = any (p.interview_types)
    ) then
        raise exception 'interviewer not available for this interview type';
    end if;

    insert into mock_interview_request (
        interviewee_user_id,
        interviewer_user_id,
        interview_type,
        message
    ) values (
        p_interviewee_user_id,
        v_interviewer_user_id,
        v_interview_type,
        nullif(trim(p_input->>'message'), '')
    )
    returning * into v_request;

    return mock_interview_request_json(v_request);
end;
$$;
