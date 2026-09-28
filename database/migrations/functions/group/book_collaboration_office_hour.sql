create or replace function book_collaboration_office_hour(
    p_user_id uuid,
    p_session_id uuid,
    p_question text
)
returns uuid as $$
declare
    v_booking_attempt integer;
    v_booking_exists boolean;
    v_member_id uuid;
    v_project_id uuid;
    v_booking_id uuid;
    v_booking_status text;
    v_capacity integer;
begin
    select
        s.collaboration_project_id,
        m.collaboration_project_member_id,
        s.capacity
    into v_project_id, v_member_id, v_capacity
    from collaboration_office_hour_session s
    join collaboration_project_member m
      on m.collaboration_project_id = s.collaboration_project_id
     and m.user_id = p_user_id
     and m.invitation_status = 'accepted'
    where s.collaboration_office_hour_session_id = p_session_id
      and s.status = 'scheduled'
      and s.starts_at > current_timestamp
    for update of s;

    if v_member_id is null then
        raise exception 'bookable session or accepted membership not found';
    end if;

    select collaboration_office_hour_booking_id, status, booking_attempt
    into v_booking_id, v_booking_status, v_booking_attempt
    from collaboration_office_hour_booking
    where collaboration_office_hour_session_id = p_session_id
      and collaboration_project_member_id = v_member_id
    for update;
    v_booking_exists := found;

    if v_booking_exists and v_booking_status = 'booked' then
        update collaboration_office_hour_booking
        set question = nullif(btrim(p_question), '')
        where collaboration_office_hour_booking_id = v_booking_id
          and question is distinct from nullif(btrim(p_question), '');
        return v_booking_id;
    end if;

    if v_booking_exists and v_booking_status <> 'cancelled' then
        raise exception 'completed office-hour booking cannot be booked again';
    end if;

    if (
        select count(*)
        from collaboration_office_hour_booking b
        where b.collaboration_office_hour_session_id = p_session_id
          and b.status = 'booked'
    ) >= v_capacity then
        raise exception 'office-hour session is at capacity';
    end if;

    if v_booking_exists then
        v_booking_attempt := v_booking_attempt + 1;
        update collaboration_office_hour_booking
        set status = 'booked',
            question = nullif(btrim(p_question), ''),
            booked_at = current_timestamp,
            cancelled_at = null,
            booking_attempt = v_booking_attempt
        where collaboration_office_hour_booking_id = v_booking_id;
    else
        insert into collaboration_office_hour_booking (
            collaboration_office_hour_session_id,
            collaboration_project_member_id, question
        ) values (
            p_session_id, v_member_id, nullif(btrim(p_question), '')
        )
        returning collaboration_office_hour_booking_id, booking_attempt
        into v_booking_id, v_booking_attempt;
    end if;

    insert into collaboration_notification_delivery (
        collaboration_project_id, recipient_user_id, kind, idempotency_key, payload
    ) values (
        v_project_id, p_user_id, 'booking',
        'booking:' || v_booking_id::text || ':' || v_booking_attempt::text,
        jsonb_build_object('booking_id', v_booking_id, 'session_id', p_session_id)
    ) on conflict do nothing;

    insert into collaboration_project_activity (
        collaboration_project_id, actor_user_id, kind, subject_type, subject_id
    ) values (v_project_id, p_user_id, 'office_hour.booked', 'booking', v_booking_id);

    return v_booking_id;
end;
$$ language plpgsql;
