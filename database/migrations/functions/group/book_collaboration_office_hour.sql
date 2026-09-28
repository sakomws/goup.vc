create or replace function book_collaboration_office_hour(
    p_user_id uuid,
    p_session_id uuid,
    p_question text
)
returns uuid as $$
declare
    v_member_id uuid;
    v_project_id uuid;
    v_booking_id uuid;
begin
    select s.collaboration_project_id, m.collaboration_project_member_id
    into v_project_id, v_member_id
    from collaboration_office_hour_session s
    join collaboration_project_member m
      on m.collaboration_project_id = s.collaboration_project_id
     and m.user_id = p_user_id
     and m.invitation_status = 'accepted'
    where s.collaboration_office_hour_session_id = p_session_id
      and s.status = 'scheduled'
      and s.starts_at > current_timestamp
      and (
        select count(*) from collaboration_office_hour_booking b
        where b.collaboration_office_hour_session_id = s.collaboration_office_hour_session_id
          and b.status = 'booked'
      ) < s.capacity
    for update of s;

    if v_member_id is null then
        raise exception 'bookable session or accepted membership not found';
    end if;

    insert into collaboration_office_hour_booking (
        collaboration_office_hour_session_id,
        collaboration_project_member_id, question
    ) values (p_session_id, v_member_id, nullif(p_question, ''))
    on conflict (collaboration_office_hour_session_id, collaboration_project_member_id)
    do update set status = 'booked', question = excluded.question, cancelled_at = null
    returning collaboration_office_hour_booking_id into v_booking_id;

    insert into collaboration_notification_delivery (
        collaboration_project_id, recipient_user_id, kind, idempotency_key, payload
    ) values (
        v_project_id, p_user_id, 'booking',
        'booking:' || v_booking_id::text,
        jsonb_build_object('booking_id', v_booking_id, 'session_id', p_session_id)
    ) on conflict do nothing;

    insert into collaboration_project_activity (
        collaboration_project_id, actor_user_id, kind, subject_type, subject_id
    ) values (v_project_id, p_user_id, 'office_hour.booked', 'booking', v_booking_id);

    return v_booking_id;
end;
$$ language plpgsql;
