-- Sends one speaker prep email and records the step after enqueue succeeds.
create or replace function send_cfs_submission_prep_email(
    p_created_by uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_cfs_submission_id uuid,
    p_step text,
    p_template_data jsonb,
    p_subject text,
    p_body text
)
returns void as $$
declare
    v_alliance_id uuid;
    v_status_id text;
    v_canceled boolean;
    v_session_id uuid;
    v_starts_at timestamptz;
    v_venue_name text;
    v_track_count int;
    v_recipients uuid[];
begin
    if p_step not in ('listed', 'profile', 'slides', 'schedule') then
        raise exception 'Choose a speaker prep email.' using errcode = 'OCG01';
    end if;

    select
        g.alliance_id,
        cs.status_id,
        e.canceled,
        s.session_id,
        s.starts_at,
        nullif(btrim(e.venue_name), '')
    into
        v_alliance_id,
        v_status_id,
        v_canceled,
        v_session_id,
        v_starts_at,
        v_venue_name
    from cfs_submission cs
    join event e on e.event_id = cs.event_id
    join "group" g on g.group_id = e.group_id
    left join session s on s.cfs_submission_id = cs.cfs_submission_id
    where cs.cfs_submission_id = p_cfs_submission_id
    and cs.event_id = p_event_id
    and e.group_id = p_group_id
    and e.deleted = false
    for update of cs;

    if not found then
        raise exception 'Submission not found.' using errcode = 'OCG01';
    end if;

    if v_canceled then
        raise exception 'Canceled events cannot send speaker emails.' using errcode = 'OCG01';
    end if;

    if v_status_id <> 'approved' then
        raise exception 'Only approved submissions can receive speaker prep emails.' using errcode = 'OCG01';
    end if;

    if exists (
        select 1
        from cfs_submission_prep_email prep
        where prep.cfs_submission_id = p_cfs_submission_id
        and prep.prep_step = p_step
    ) then
        raise exception 'This speaker prep email was already sent.' using errcode = 'OCG01';
    end if;

    if p_step = 'listed' and v_session_id is null then
        raise exception 'Add this talk to the agenda before emailing that it is listed.' using errcode = 'OCG01';
    end if;

    if p_step = 'schedule' then
        select count(*)
        into v_track_count
        from cfs_submission_label
        where cfs_submission_id = p_cfs_submission_id;

        if v_track_count = 0 then
            raise exception 'Assign a track before emailing the schedule.' using errcode = 'OCG01';
        end if;
        if v_starts_at is null then
            raise exception 'Set the session time before emailing the schedule.' using errcode = 'OCG01';
        end if;
        if v_venue_name is null then
            raise exception 'Add the event venue before emailing the schedule.' using errcode = 'OCG01';
        end if;
    end if;

    select coalesce(array_agg(recipients.user_id order by recipients.user_id asc), array[]::uuid[])
    into v_recipients
    from (
        select distinct person.user_id
        from cfs_submission cs
        join session_proposal sp on sp.session_proposal_id = cs.session_proposal_id
        cross join lateral (values (sp.user_id), (sp.co_speaker_user_id)) as person(user_id)
        join "user" u on u.user_id = person.user_id
        where cs.cfs_submission_id = p_cfs_submission_id
        and person.user_id is not null
        and u.email_verified = true
        and exists (
            select 1
            from users_accepting_notification('event-custom', array[person.user_id], array[p_group_id])
        )
    ) recipients;

    if coalesce(array_length(v_recipients, 1), 0) = 0 then
        raise exception 'No speakers with verified email addresses who accept messages from event organizers.'
            using errcode = 'OCG01';
    end if;

    perform enqueue_tracked_custom_notification(
        'event-custom',
        p_template_data,
        '[]'::jsonb,
        v_recipients,
        p_created_by,
        v_alliance_id,
        p_event_id,
        p_group_id,
        coalesce(array_length(v_recipients, 1), 0),
        p_subject,
        p_body,
        null
    );

    insert into cfs_submission_prep_email (cfs_submission_id, prep_step, sent_by)
    values (p_cfs_submission_id, p_step, p_created_by);
end;
$$ language plpgsql;
