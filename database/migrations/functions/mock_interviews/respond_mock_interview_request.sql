create or replace function respond_mock_interview_request(
    p_actor_user_id uuid,
    p_request_id uuid,
    p_action text,
    p_meeting_url text default null
)
returns jsonb language plpgsql as $$
declare
    v_request mock_interview_request;
    v_session mock_interview_session;
begin
    select * into v_request
    from mock_interview_request
    where mock_interview_request_id = p_request_id
    for update;

    if v_request.mock_interview_request_id is null then
        raise exception 'request not found';
    end if;

    if p_action = 'accept' then
        if v_request.interviewer_user_id <> p_actor_user_id then
            raise exception 'only the interviewer can accept';
        end if;
        if v_request.status <> 'pending' then
            raise exception 'request is not pending';
        end if;

        update mock_interview_request
        set status = 'accepted',
            responded_at = current_timestamp
        where mock_interview_request_id = p_request_id
        returning * into v_request;

        insert into mock_interview_session (
            mock_interview_request_id,
            meeting_url,
            status
        ) values (
            p_request_id,
            nullif(trim(p_meeting_url), ''),
            'scheduled'
        )
        returning * into v_session;

        return jsonb_build_object(
            'request', mock_interview_request_json(v_request),
            'session_id', v_session.mock_interview_session_id
        );
    elsif p_action = 'decline' then
        if v_request.interviewer_user_id <> p_actor_user_id then
            raise exception 'only the interviewer can decline';
        end if;
        if v_request.status <> 'pending' then
            raise exception 'request is not pending';
        end if;

        update mock_interview_request
        set status = 'declined',
            responded_at = current_timestamp
        where mock_interview_request_id = p_request_id
        returning * into v_request;

        return jsonb_build_object('request', mock_interview_request_json(v_request));
    elsif p_action = 'cancel' then
        if v_request.interviewee_user_id <> p_actor_user_id then
            raise exception 'only the interviewee can cancel';
        end if;
        if v_request.status not in ('pending', 'accepted') then
            raise exception 'request cannot be cancelled';
        end if;

        update mock_interview_request
        set status = 'cancelled',
            responded_at = current_timestamp
        where mock_interview_request_id = p_request_id
        returning * into v_request;

        update mock_interview_session
        set status = 'cancelled'
        where mock_interview_request_id = p_request_id;

        return jsonb_build_object('request', mock_interview_request_json(v_request));
    else
        raise exception 'unsupported action';
    end if;
end;
$$;
