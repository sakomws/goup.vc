create or replace function suggest_gtm_lead_candidates(
    p_alliance_id uuid,
    p_group_id uuid,
    p_limit int
) returns jsonb language plpgsql stable as $$
declare
    v_limit int := least(coalesce(p_limit, 12), 25);
    v_candidates jsonb;
begin
    with landscape_candidates as (
        select
            le.name,
            le.name as org_name,
            null::text as email,
            le.website_url,
            null::text as linkedin_url,
            case
                when le.kind = 'investor' then 'investor'
                when le.kind = 'startup' then 'startup'
                else 'sponsor'
            end as kind,
            'landscape'::text as source,
            le.landscape_entry_id,
            null::uuid as user_id,
            le.summary as notes
        from landscape_entry le
        where le.alliance_id = p_alliance_id
          and le.kind in ('startup', 'investor', 'partner_community')
          and le.published = true
          and not exists (
              select 1 from gtm_lead gl
              where gl.alliance_id = p_alliance_id
                and (
                    gl.landscape_entry_id = le.landscape_entry_id
                    or (
                        le.website_url is not null
                        and gl.website_url is not null
                        and lower(gl.website_url) = lower(le.website_url)
                    )
                )
          )
        order by coalesce(le.updated_at, le.created_at) desc
        limit v_limit
    ),
    member_candidates as (
        select
            coalesce(u.name, u.username) as name,
            nullif(u.company, '') as org_name,
            u.email,
            u.website_url,
            u.linkedin_url,
            'organizer'::text as kind,
            'member'::text as source,
            null::uuid as landscape_entry_id,
            u.user_id,
            nullif(u.title, '') as notes
        from group_member gm
        join "user" u using (user_id)
        join "group" g on g.group_id = gm.group_id
        where g.alliance_id = p_alliance_id
          and g.deleted = false
          and (p_group_id is null or gm.group_id = p_group_id)
          and u.email is not null
          and not exists (
              select 1 from gtm_lead gl
              where gl.alliance_id = p_alliance_id
                and (
                    gl.user_id = u.user_id
                    or lower(gl.email) = lower(u.email)
                )
          )
        order by gm.created_at desc
        limit v_limit
    ),
    sponsor_candidates as (
        select
            gs.name,
            gs.name as org_name,
            null::text as email,
            gs.website_url,
            null::text as linkedin_url,
            'sponsor'::text as kind,
            'manual'::text as source,
            null::uuid as landscape_entry_id,
            null::uuid as user_id,
            'Existing group sponsor'::text as notes
        from group_sponsor gs
        join "group" g using (group_id)
        where g.alliance_id = p_alliance_id
          and (p_group_id is null or gs.group_id = p_group_id)
          and not exists (
              select 1 from gtm_lead gl
              where gl.alliance_id = p_alliance_id
                and (
                    gl.group_sponsor_id = gs.group_sponsor_id
                    or (
                        gs.website_url is not null
                        and gl.website_url is not null
                        and lower(gl.website_url) = lower(gs.website_url)
                    )
                )
          )
        order by gs.created_at desc
        limit v_limit
    ),
    combined as (
        select * from landscape_candidates
        union all
        select * from member_candidates
        union all
        select * from sponsor_candidates
    )
    select coalesce(jsonb_agg(jsonb_strip_nulls(jsonb_build_object(
        'name', name,
        'org_name', org_name,
        'email', email,
        'website_url', website_url,
        'linkedin_url', linkedin_url,
        'kind', kind,
        'source', source,
        'landscape_entry_id', landscape_entry_id,
        'user_id', user_id,
        'notes', notes,
        'group_id', p_group_id
    ))), '[]'::jsonb)
    into v_candidates
    from (
        select * from combined limit v_limit
    ) candidates;

    return jsonb_build_object('candidates', coalesce(v_candidates, '[]'::jsonb));
end;
$$;
