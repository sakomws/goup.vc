-- list_landscape_entry_options returns published landscape entries users can
-- declare an affiliation with.
create or replace function list_landscape_entry_options()
returns json as $$
    select coalesce(json_agg(json_build_object(
        'landscape_entry_id', landscape_entry_id,
        'name', name,
        'kind', kind
    ) order by kind, lower(name)), '[]'::json)
    from landscape_entry
    where published = true;
$$ language sql stable;
