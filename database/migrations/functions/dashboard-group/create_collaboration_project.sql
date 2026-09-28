create or replace function create_collaboration_project(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_project jsonb
)
returns uuid as $$
declare
    v_project_id uuid;
begin
    insert into collaboration_project (
        group_id, created_by, landscape_entry_id, group_accelerator_cohort_id,
        slug, name, summary, description, lifecycle, visibility, website_url,
        repository_url, cover_image_url, starts_on, target_ends_on
    ) values (
        p_group_id, p_actor_user_id,
        (p_project->>'landscape_entry_id')::uuid,
        (p_project->>'group_accelerator_cohort_id')::uuid,
        p_project->>'slug', p_project->>'name', p_project->>'summary',
        nullif(p_project->>'description', ''),
        coalesce(nullif(p_project->>'lifecycle', ''), 'proposed'),
        coalesce(nullif(p_project->>'visibility', ''), 'members'),
        nullif(p_project->>'website_url', ''),
        nullif(p_project->>'repository_url', ''),
        nullif(p_project->>'cover_image_url', ''),
        (p_project->>'starts_on')::date,
        (p_project->>'target_ends_on')::date
    )
    returning collaboration_project_id into v_project_id;

    insert into collaboration_project_member (
        collaboration_project_id, user_id, role, invitation_status,
        invited_by, responded_at
    ) values (
        v_project_id, p_actor_user_id, 'owner', 'accepted',
        p_actor_user_id, current_timestamp
    );

    insert into collaboration_project_activity (
        collaboration_project_id, actor_user_id, kind, subject_type, subject_id
    ) values (v_project_id, p_actor_user_id, 'project.created', 'project', v_project_id);

    return v_project_id;
end;
$$ language plpgsql;
