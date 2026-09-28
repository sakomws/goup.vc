create or replace function collaboration_project_json(p collaboration_project)
returns jsonb as $$
    select jsonb_build_object(
        'collaboration_project_id', p.collaboration_project_id,
        'group_id', p.group_id,
        'slug', p.slug,
        'name', p.name,
        'summary', p.summary,
        'description', p.description,
        'lifecycle', p.lifecycle,
        'visibility', p.visibility,
        'landscape_entry_id', p.landscape_entry_id,
        'group_accelerator_cohort_id', p.group_accelerator_cohort_id,
        'website_url', p.website_url,
        'repository_url', p.repository_url,
        'cover_image_url', p.cover_image_url,
        'starts_on', p.starts_on,
        'target_ends_on', p.target_ends_on,
        'created_at', extract(epoch from p.created_at)::bigint,
        'updated_at', extract(epoch from p.updated_at)::bigint
    );
$$ language sql immutable;
