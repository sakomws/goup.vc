-- Durable organizer-created event attendee emails. A scheduled message stores
-- the fully rendered template payload, while recipients are re-checked when it
-- becomes due so opt-outs and canceled registrations are always respected.
create table if not exists scheduled_event_attendee_email (
    scheduled_event_attendee_email_id uuid primary key default gen_random_uuid(),
    event_id uuid not null references event (event_id) on delete cascade,
    group_id uuid not null references "group" (group_id) on delete cascade,
    created_by uuid references "user" (user_id) on delete set null,
    subject text not null check (btrim(subject) <> ''),
    body text not null check (btrim(body) <> ''),
    template_data jsonb not null,
    recipient_scope text not null check (recipient_scope in ('all-attendees', 'selected-attendees')),
    recipient_user_ids uuid[],
    recipient_count integer not null check (recipient_count >= 0),
    scheduled_at timestamptz not null,
    status text not null default 'scheduled' check (status in ('scheduled', 'sent', 'canceled')),
    sent_at timestamptz,
    canceled_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp,
    check (
        (recipient_scope = 'all-attendees' and recipient_user_ids is null)
        or (recipient_scope = 'selected-attendees' and cardinality(recipient_user_ids) > 0)
    ),
    check (
        (status = 'scheduled' and sent_at is null and canceled_at is null)
        or (status = 'sent' and sent_at is not null and canceled_at is null)
        or (status = 'canceled' and canceled_at is not null and sent_at is null)
    )
);

create index if not exists scheduled_event_attendee_email_due_idx
    on scheduled_event_attendee_email (scheduled_at, scheduled_event_attendee_email_id)
    where status = 'scheduled';

create index if not exists scheduled_event_attendee_email_event_idx
    on scheduled_event_attendee_email (event_id, created_at desc);

create or replace function schedule_event_attendee_email(
    p_created_by uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_subject text,
    p_body text,
    p_template_data jsonb,
    p_recipient_scope text,
    p_recipient_user_ids uuid[],
    p_recipient_count integer,
    p_scheduled_at timestamptz
)
returns uuid as $$
declare
    v_scheduled_event_attendee_email_id uuid;
begin
    if not exists (
        select 1
        from event
        where event_id = p_event_id
        and group_id = p_group_id
        and canceled = false
    ) then
        raise exception 'event not found or unavailable';
    end if;

    insert into scheduled_event_attendee_email (
        body,
        created_by,
        event_id,
        group_id,
        recipient_count,
        recipient_scope,
        recipient_user_ids,
        scheduled_at,
        subject,
        template_data
    )
    values (
        p_body,
        p_created_by,
        p_event_id,
        p_group_id,
        p_recipient_count,
        p_recipient_scope,
        p_recipient_user_ids,
        p_scheduled_at,
        p_subject,
        p_template_data
    )
    returning scheduled_event_attendee_email_id into v_scheduled_event_attendee_email_id;

    return v_scheduled_event_attendee_email_id;
end;
$$ language plpgsql;

create or replace function update_scheduled_event_attendee_email(
    p_scheduled_event_attendee_email_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_subject text,
    p_body text,
    p_template_data jsonb,
    p_recipient_scope text,
    p_recipient_user_ids uuid[],
    p_recipient_count integer,
    p_scheduled_at timestamptz
)
returns void as $$
begin
    update scheduled_event_attendee_email
    set
        body = p_body,
        recipient_count = p_recipient_count,
        recipient_scope = p_recipient_scope,
        recipient_user_ids = p_recipient_user_ids,
        scheduled_at = p_scheduled_at,
        subject = p_subject,
        template_data = p_template_data,
        updated_at = current_timestamp
    where scheduled_event_attendee_email_id = p_scheduled_event_attendee_email_id
    and event_id = p_event_id
    and group_id = p_group_id
    and status = 'scheduled';

    if not found then
        raise exception 'scheduled attendee email not found or cannot be updated';
    end if;
end;
$$ language plpgsql;

create or replace function cancel_scheduled_event_attendee_email(
    p_scheduled_event_attendee_email_id uuid,
    p_group_id uuid,
    p_event_id uuid
)
returns void as $$
begin
    update scheduled_event_attendee_email
    set
        canceled_at = current_timestamp,
        status = 'canceled',
        updated_at = current_timestamp
    where scheduled_event_attendee_email_id = p_scheduled_event_attendee_email_id
    and event_id = p_event_id
    and group_id = p_group_id
    and status = 'scheduled';

    if not found then
        raise exception 'scheduled attendee email not found or cannot be canceled';
    end if;
end;
$$ language plpgsql;

create or replace function list_scheduled_event_attendee_emails(
    p_group_id uuid,
    p_event_id uuid
)
returns jsonb as $$
    select coalesce(
        jsonb_agg(
            jsonb_build_object(
                'body', see.body,
                'recipient_count', see.recipient_count,
                'recipient_scope', see.recipient_scope,
                'recipient_user_ids', see.recipient_user_ids,
                'scheduled_at', extract(epoch from see.scheduled_at)::bigint,
                'scheduled_event_attendee_email_id', see.scheduled_event_attendee_email_id,
                'status', see.status,
                'subject', see.subject
            )
            order by see.scheduled_at asc, see.created_at asc
        ),
        '[]'::jsonb
    )
    from scheduled_event_attendee_email see
    where see.event_id = p_event_id
    and see.group_id = p_group_id;
$$ language sql stable;

create or replace function enqueue_due_scheduled_event_attendee_emails()
returns integer as $$
declare
    v_schedule scheduled_event_attendee_email%rowtype;
    v_recipients uuid[];
    v_count integer := 0;
begin
    for v_schedule in
        select *
        from scheduled_event_attendee_email
        where status = 'scheduled'
        and scheduled_at <= current_timestamp
        order by scheduled_at asc, scheduled_event_attendee_email_id asc
        for update skip locked
    loop
        select resolve_event_custom_notification_recipient_ids(
            v_schedule.group_id,
            v_schedule.event_id,
            v_schedule.recipient_scope,
            v_schedule.recipient_user_ids
        )
        into v_recipients;

        if cardinality(v_recipients) > 0 then
            perform enqueue_notification(
                'event-custom',
                v_schedule.template_data,
                '[]'::jsonb,
                v_recipients
            );
            perform track_custom_notification(
                v_schedule.created_by,
                v_schedule.event_id,
                v_schedule.group_id,
                cardinality(v_recipients),
                v_schedule.subject,
                v_schedule.body
            );
        end if;

        update scheduled_event_attendee_email
        set
            recipient_count = coalesce(cardinality(v_recipients), 0),
            sent_at = current_timestamp,
            status = 'sent',
            updated_at = current_timestamp
        where scheduled_event_attendee_email_id = v_schedule.scheduled_event_attendee_email_id;
        v_count := v_count + 1;
    end loop;

    return v_count;
end;
$$ language plpgsql;
