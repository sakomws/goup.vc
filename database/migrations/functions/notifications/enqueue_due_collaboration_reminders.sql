create or replace function enqueue_due_collaboration_reminders(p_now timestamptz default current_timestamp)
returns integer as $$
declare
    v_inserted integer;
begin
    with due as (
        select
            p.collaboration_project_id,
            m.user_id,
            s.collaboration_office_hour_session_id,
            b.collaboration_office_hour_booking_id,
            b.booking_attempt
        from collaboration_office_hour_booking b
        join collaboration_office_hour_session s using (collaboration_office_hour_session_id)
        join collaboration_project p using (collaboration_project_id)
        join collaboration_project_member m using (collaboration_project_member_id)
        where b.status = 'booked'
          and m.invitation_status = 'accepted'
          and s.status = 'scheduled'
          and s.starts_at > p_now
          and s.starts_at <= p_now + interval '24 hours'
    ),
    inserted as (
        insert into collaboration_notification_delivery (
            collaboration_project_id, recipient_user_id, kind, idempotency_key, payload
        )
        select
            collaboration_project_id,
            user_id,
            'session_reminder',
            'session-reminder:' || collaboration_office_hour_booking_id::text
                || ':' || booking_attempt::text,
            jsonb_build_object(
                'booking_id', collaboration_office_hour_booking_id,
                'session_id', collaboration_office_hour_session_id
            )
        from due
        on conflict do nothing
        returning 1
    )
    select count(*) into v_inserted from inserted;

    return v_inserted;
end;
$$ language plpgsql;
