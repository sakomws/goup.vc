-- Describes alliance contact filters with readable names for audit details.
-- Empty or missing filter lists are described as "All", and selected groups
-- by their contact filter labels. Callers validate the filters with
-- validate_alliance_contact_filters first.
create or replace function describe_alliance_contact_filters(
    p_alliance_id uuid,
    p_filters jsonb
)
returns jsonb as $$
    -- Build the readable names for each filter
    select jsonb_build_object(
        'group_categories', case
            when cardinality(f.group_category_ids) = 0 then to_jsonb('All'::text)
            else (
                select coalesce(jsonb_agg(gc.name order by gc."order" nulls last, gc.name), '[]')
                from group_category gc
                where gc.alliance_id = p_alliance_id
                and gc.group_category_id = any(f.group_category_ids)
            )
        end,
        'groups', case
            when cardinality(f.group_ids) = 0 then to_jsonb('All'::text)
            else (
                select coalesce(jsonb_agg(l.label order by l.label, l.group_id), '[]')
                from alliance_contact_group_labels(p_alliance_id) l
                where l.group_id = any(f.group_ids)
            )
        end,
        'regions', case
            when cardinality(f.region_ids) = 0 and not f.include_no_region then to_jsonb('All'::text)
            else (
                select coalesce(jsonb_agg(r.name order by r."order" nulls last, r.name), '[]')
                from region r
                where r.alliance_id = p_alliance_id
                and r.region_id = any(f.region_ids)
            ) || case when f.include_no_region then '["No region"]'::jsonb else '[]'::jsonb end
        end,
        'roles', case
            when cardinality(f.roles) = 0 then to_jsonb('All'::text)
            else (
                select coalesce(
                    jsonb_agg(
                        gr.display_name
                        order by
                            array_position(
                                array['admin', 'events-manager', 'check-in-manager', 'viewer'],
                                gr.group_role_id
                            ) nulls last,
                            gr.display_name
                    ),
                    '[]'
                )
                from group_role gr
                where gr.group_role_id = any(f.roles)
            )
        end
    )
    from alliance_contact_filter_values(p_filters) f;
$$ language sql stable;
