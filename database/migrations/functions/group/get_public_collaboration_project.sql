create or replace function get_public_collaboration_project(
    p_alliance_id uuid,
    p_group_slug text,
    p_project_slug text
)
returns jsonb as $$
    select collaboration_project_json(p) || jsonb_build_object(
        'goals', coalesce((
            select jsonb_agg(jsonb_build_object(
                'title', goal.title, 'description', goal.description,
                'status', goal.status, 'target_value', goal.target_value,
                'current_value', goal.current_value, 'unit', goal.unit,
                'due_on', goal.due_on
            ) order by goal.created_at)
            from collaboration_project_goal goal
            where goal.collaboration_project_id = p.collaboration_project_id
        ), '[]'::jsonb),
        'updates', coalesce((
            select jsonb_agg(jsonb_build_object(
                'body', update.body,
                'next_steps', update.next_steps,
                'created_at', extract(epoch from update.created_at)::bigint
            ) order by update.created_at, update.collaboration_project_update_id)
            from collaboration_project_update update
            where update.collaboration_project_id = p.collaboration_project_id
        ), '[]'::jsonb),
        'outcome_summary', get_collaboration_outcome_summary(p.collaboration_project_id)
    )
    from collaboration_project p
    join "group" g using (group_id)
    where g.alliance_id = p_alliance_id
      and (g.slug = p_group_slug or g.group_id::text = p_group_slug)
      and p.slug = p_project_slug
      and p.visibility = 'public'
      and p.lifecycle <> 'archived';
$$ language sql stable;
