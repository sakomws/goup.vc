-- Returns the speaker prep facts used to compose and guard one submission email.
create or replace function get_cfs_submission_prep_email_context(
    p_group_id uuid,
    p_event_id uuid,
    p_cfs_submission_id uuid
)
returns json as $$
    select json_strip_nulls(json_build_object(
        'co_speaker_user_id', sp.co_speaker_user_id,
        'linked_session_id', s.session_id,
        'prep_emails', coalesce((
            select json_agg(prep.prep_step order by prep.created_at asc, prep.prep_step asc)
            from cfs_submission_prep_email prep
            where prep.cfs_submission_id = cs.cfs_submission_id
        ), '[]'::json),
        'session_starts_at', epoch_seconds(s.starts_at),
        'speaker_user_id', sp.user_id,
        'status_id', cs.status_id,
        'title', sp.title,
        'tracks', coalesce((
            select json_agg(ecl.name order by ecl.name asc, ecl.event_cfs_label_id asc)
            from cfs_submission_label csl
            join event_cfs_label ecl on ecl.event_cfs_label_id = csl.event_cfs_label_id
            where csl.cfs_submission_id = cs.cfs_submission_id
        ), '[]'::json),
        'venue_name', nullif(btrim(e.venue_name), '')
    ))
    from cfs_submission cs
    join event e on e.event_id = cs.event_id
    join session_proposal sp on sp.session_proposal_id = cs.session_proposal_id
    left join session s on s.cfs_submission_id = cs.cfs_submission_id
    where cs.cfs_submission_id = p_cfs_submission_id
    and cs.event_id = p_event_id
    and e.group_id = p_group_id
    and e.deleted = false;
$$ language sql;
