-- list_user_affiliations returns the landscape entries the user is affiliated with.
create or replace function list_user_affiliations(p_user_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'user_affiliation_id', ua.user_affiliation_id,
        'landscape_entry_id', le.landscape_entry_id,
        'entry_name', le.name,
        'entry_kind', le.kind,
        'entry_logo_url', le.logo_url,
        'entry_website_url', le.website_url,
        'entry_github_url', le.github_url,
        'role', ua.role
    ) order by le.kind, lower(le.name)), '[]'::json)
    from user_affiliation ua
    join landscape_entry le using (landscape_entry_id)
    where ua.user_id = p_user_id;
$$ language sql stable;
