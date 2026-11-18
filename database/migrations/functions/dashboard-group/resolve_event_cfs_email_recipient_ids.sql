-- Resolves speakers and co-speakers who can receive an organizer email for a submission audience.
-- Withdrawn submissions are excluded. Audiences are approved, rejected, and all other active statuses.
create or replace function resolve_event_cfs_email_recipient_ids(
    p_group_id uuid,
    p_event_id uuid,
    p_audience text
)
returns uuid[] as $$
    select coalesce(array_agg(recipients.user_id order by recipients.user_id asc), array[]::uuid[])
    from (
        select distinct person.user_id
        from cfs_submission cs
        join event e on e.event_id = cs.event_id
        join session_proposal sp on sp.session_proposal_id = cs.session_proposal_id
        cross join lateral (
            values (sp.user_id), (sp.co_speaker_user_id)
        ) as person(user_id)
        join "user" u on u.user_id = person.user_id
        where cs.event_id = p_event_id
        and e.group_id = p_group_id
        and person.user_id is not null
        and cs.status_id <> 'withdrawn'
        and (
            (p_audience = 'all')
            or (p_audience = 'approved' and cs.status_id = 'approved')
            or (p_audience = 'rejected' and cs.status_id = 'rejected')
        )
        and u.email_verified = true
        and exists (
            select 1
            from users_accepting_notification('event-custom', array[person.user_id], array[p_group_id])
        )
    ) recipients;
$$ language sql;
