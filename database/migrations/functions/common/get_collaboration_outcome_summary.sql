create or replace function get_collaboration_outcome_summary(p_project_id uuid)
returns jsonb as $$
    select jsonb_build_object(
        'total', count(*),
        'measured', count(*) filter (where achieved_value is not null),
        'targets_met', count(*) filter (
            where achieved_value is not null
              and target_value is not null
              and achieved_value >= target_value
        ),
        'outcomes', coalesce(jsonb_agg(jsonb_build_object(
            'title', title,
            'narrative', narrative,
            'metric_name', metric_name,
            'baseline_value', baseline_value,
            'target_value', target_value,
            'achieved_value', achieved_value,
            'unit', unit,
            'measured_on', measured_on,
            'evidence', coalesce((
                select jsonb_agg(jsonb_build_object('label', e.label, 'url', e.url) order by e.created_at)
                from collaboration_project_evidence e
                where e.collaboration_project_outcome_id = o.collaboration_project_outcome_id
            ), '[]'::jsonb)
        ) order by created_at desc) filter (where collaboration_project_outcome_id is not null), '[]'::jsonb)
    )
    from collaboration_project_outcome o
    where collaboration_project_id = p_project_id;
$$ language sql stable;
