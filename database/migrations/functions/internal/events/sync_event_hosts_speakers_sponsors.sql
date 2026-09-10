-- sync_event_hosts_speakers_sponsors synchronizes an event's hosts, speakers, and sponsors.
create or replace function sync_event_hosts_speakers_sponsors(
    p_event_id uuid,
    p_event jsonb
)
returns void as $$
declare
    v_event_group_id uuid;
    v_group_sponsor_id uuid;
    v_sponsor jsonb;
begin
    select group_id
    into v_event_group_id
    from event
    where event_id = p_event_id;

    -- Reject reusable sponsors from another group and event-scoped sponsors
    -- from any other event.
    if exists (
        select 1
        from jsonb_to_recordset(coalesce(p_event->'sponsors', '[]'::jsonb))
            as sponsor(group_sponsor_id uuid)
        join group_sponsor gs on gs.group_sponsor_id = sponsor.group_sponsor_id
        where gs.group_id <> v_event_group_id
           or (gs.event_id is not null and gs.event_id <> p_event_id)
    ) then
        raise exception 'sponsor is not available for this event';
    end if;

    -- Replace associations from the payload
    delete from event_host where event_id = p_event_id;
    delete from event_speaker where event_id = p_event_id;
    delete from event_sponsor where event_id = p_event_id;

    -- Remove event-only sponsor profiles that are no longer submitted.
    delete from group_sponsor gs
    where gs.event_id = p_event_id
      and not exists (
          select 1
          from jsonb_to_recordset(coalesce(p_event->'sponsors', '[]'::jsonb))
              as sponsor(group_sponsor_id uuid)
          where sponsor.group_sponsor_id = gs.group_sponsor_id
      );

    if p_event->'hosts' is not null then
        insert into event_host (event_id, user_id)
        select p_event_id, host.user_id::uuid
        from jsonb_array_elements_text(p_event->'hosts') as host(user_id);
    end if;

    if p_event->'speakers' is not null then
        insert into event_speaker (event_id, user_id, featured)
        select p_event_id, speaker.user_id, speaker.featured
        from jsonb_to_recordset(p_event->'speakers') as speaker(featured boolean, user_id uuid);
    end if;

    if p_event->'sponsors' is not null then
        for v_sponsor in
            select value
            from jsonb_array_elements(p_event->'sponsors')
        loop
            v_group_sponsor_id := nullif(v_sponsor->>'group_sponsor_id', '')::uuid;

            if v_group_sponsor_id is null then
                if nullif(btrim(v_sponsor->>'name'), '') is null
                   or nullif(btrim(v_sponsor->>'logo_url'), '') is null then
                    raise exception 'event-only sponsor requires name and logo URL';
                end if;

                insert into group_sponsor (
                    event_id,
                    featured,
                    group_id,
                    logo_url,
                    name,
                    website_url
                ) values (
                    p_event_id,
                    false,
                    v_event_group_id,
                    v_sponsor->>'logo_url',
                    v_sponsor->>'name',
                    nullif(v_sponsor->>'website_url', '')
                )
                returning group_sponsor_id into v_group_sponsor_id;
            end if;

            insert into event_sponsor (event_id, group_sponsor_id, level)
            values (p_event_id, v_group_sponsor_id, v_sponsor->>'level');
        end loop;
    end if;
end;
$$ language plpgsql;
