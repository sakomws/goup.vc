-- Synchronizes approved CFS submission speakers (including co-speakers) into event_speaker.
create or replace function sync_event_approved_speakers(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid
)
returns json as $$
declare
    v_alliance_id uuid;
    v_inserted_count integer := 0;
begin
    -- Ensure the event exists, belongs to the group, and is editable
    select g.alliance_id
    into v_alliance_id
    from event e
    join "group" g using (group_id)
    where e.event_id = p_event_id
    and e.group_id = p_group_id
    and e.deleted = false
    and e.canceled = false
    for update of e;

    if not found then
        raise exception 'event not found';
    end if;

    -- Insert missing primary and co-speaker users from approved CFS submissions
    with approved_speakers as (
        select distinct speaker_user_id as user_id
        from cfs_submission cs
        join session_proposal sp on sp.session_proposal_id = cs.session_proposal_id
        cross join lateral (
            values (sp.user_id), (sp.co_speaker_user_id)
        ) as speaker(speaker_user_id)
        where cs.event_id = p_event_id
        and cs.status_id = 'approved'
        and speaker_user_id is not null
    ),
    inserted as (
        insert into event_speaker (event_id, user_id, featured)
        select p_event_id, user_id, false
        from approved_speakers
        on conflict (event_id, user_id) do nothing
        returning user_id
    )
    select count(*)::integer
    into v_inserted_count
    from inserted;

    if v_inserted_count > 0 then
        -- Mark automatic event meetings out of sync when speaker hosts change
        update event
        set meeting_in_sync = case
            when meeting_requested = true then false
            else meeting_in_sync
        end
        where event_id = p_event_id
        and group_id = p_group_id;

        -- Record the speaker synchronization in the audit log
        perform insert_audit_log(
            'event_updated',
            p_actor_user_id,
            'event',
            p_event_id,
            v_alliance_id,
            p_group_id,
            p_event_id
        );
    end if;

    return json_build_object(
        'added_count', v_inserted_count,
        'speakers', (
            select coalesce(json_agg(json_strip_nulls(json_build_object(
                'user_id', u.user_id,
                'username', u.username,

                'bio', u.bio,
                'bluesky_url', u.bluesky_url,
                'company', u.company,
                'facebook_url', u.facebook_url,
                'github_url', u.github_url,
                'featured', es.featured,
                'linkedin_url', u.linkedin_url,
                'name', u.name,
                'photo_url', u.photo_url,
                'provider', u.provider,
                'substack_url', u.substack_url,
                'title', u.title,
                'twitter_url', u.twitter_url,
                'website_url', u.website_url,
                'youtube_url', u.youtube_url
            )) order by es.featured desc, u.name), '[]')
            from event_speaker es
            join "user" u using (user_id)
            where es.event_id = p_event_id
        )
    );
end;
$$ language plpgsql;
