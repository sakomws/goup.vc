-- Returns private contact fields only from the admin-protected CSV export route.
create or replace function list_alliance_members_for_export(p_alliance_id uuid, p_filters jsonb)
returns jsonb language sql stable as $$
    with member_groups as (
        select gm.user_id, array_agg(distinct g.name order by g.name) as group_names
        from group_member gm
        join "group" g using (group_id)
        where g.alliance_id = p_alliance_id
          and g.active = true
          and g.deleted = false
        group by gm.user_id
    )
    select coalesce(jsonb_agg(jsonb_build_object(
        'name', u.name,
        'username', u.username,
        'email', u.email,
        'phone_country_code', u.phone_country_code,
        'phone_number', u.phone_number,
        'group_names', mg.group_names,
        'company', u.company,
        'title', u.title,
        'city', u.city,
        'country', u.country,
        'linkedin_url', u.linkedin_url,
        'github_url', u.github_url,
        'website_url', u.website_url
    ) order by lower(coalesce(u.name, u.username)), lower(u.username)), '[]'::jsonb)
    from member_groups mg
    join "user" u using (user_id)
    where u.email_verified = true
      and u.registration_status = 'registered'
      and (
          nullif(trim(p_filters->>'query'), '') is null
          or concat_ws(' ', u.username, u.name, u.email, u.company, u.title, u.city, u.country,
              array_to_string(mg.group_names, ' '))
              ilike '%' || escape_ilike_pattern(nullif(trim(p_filters->>'query'), '')) || '%' escape '\'
      );
$$;
