create or replace function create_collaboration_project(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_project jsonb
)
returns uuid as $$
declare
    v_alliance_id uuid;
    v_lifecycle text;
    v_project_id uuid;
begin
    select alliance_id
    into v_alliance_id
    from "group"
    where group_id = p_group_id
      and active = true
      and deleted = false;

    if v_alliance_id is null or not user_has_group_permission(
        v_alliance_id,
        p_group_id,
        p_actor_user_id,
        'group.projects.write'
    ) then
        raise exception 'project management permission required'
            using errcode = 'insufficient_privilege';
    end if;

    if p_project->>'landscape_entry_id' is not null and not exists (
        select 1
        from landscape_entry
        where landscape_entry_id = (p_project->>'landscape_entry_id')::uuid
          and alliance_id = v_alliance_id
    ) then
        raise exception 'landscape entry must belong to the group alliance';
    end if;

    if p_project->>'group_accelerator_cohort_id' is not null and not exists (
        select 1
        from group_accelerator_cohort c
        join group_accelerator_program p using (group_accelerator_program_id)
        where c.group_accelerator_cohort_id = (p_project->>'group_accelerator_cohort_id')::uuid
          and p.group_id = p_group_id
    ) then
        raise exception 'accelerator cohort must belong to the group';
    end if;

    v_lifecycle := coalesce(nullif(p_project->>'lifecycle', ''), 'proposed');

    insert into collaboration_project (
        group_id, created_by, landscape_entry_id, group_accelerator_cohort_id,
        slug, name, summary, description, lifecycle, visibility, website_url,
        repository_url, cover_image_url, starts_on, target_ends_on, completed_at
    ) values (
        p_group_id, p_actor_user_id,
        (p_project->>'landscape_entry_id')::uuid,
        (p_project->>'group_accelerator_cohort_id')::uuid,
        btrim(p_project->>'slug'), btrim(p_project->>'name'), btrim(p_project->>'summary'),
        nullif(p_project->>'description', ''),
        v_lifecycle,
        coalesce(nullif(p_project->>'visibility', ''), 'members'),
        nullif(p_project->>'website_url', ''),
        nullif(p_project->>'repository_url', ''),
        nullif(p_project->>'cover_image_url', ''),
        (p_project->>'starts_on')::date,
        (p_project->>'target_ends_on')::date,
        case when v_lifecycle = 'completed' then current_timestamp end
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
