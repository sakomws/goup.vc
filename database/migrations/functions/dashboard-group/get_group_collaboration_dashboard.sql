create or replace function get_group_collaboration_dashboard(
    p_group_id uuid,
    p_actor_user_id uuid
)
returns jsonb as $$
    select jsonb_build_object(
        'projects', coalesce((
            select jsonb_agg(collaboration_project_json(p) order by p.updated_at desc)
            from collaboration_project p where p.group_id = p_group_id
        ), '[]'::jsonb),
        'members', coalesce((
            select jsonb_agg(jsonb_build_object(
                'collaboration_project_member_id', m.collaboration_project_member_id,
                'collaboration_project_id', m.collaboration_project_id,
                'user_id', m.user_id,
                'role', m.role,
                'invitation_status', m.invitation_status,
                'name', u.name,
                'username', u.username
            ) order by u.name)
            from collaboration_project_member m
            join collaboration_project p using (collaboration_project_id)
            join "user" u using (user_id)
            where p.group_id = p_group_id
        ), '[]'::jsonb),
        'goals', coalesce((
            select jsonb_agg(to_jsonb(g) order by g.created_at)
            from collaboration_project_goal g
            join collaboration_project p using (collaboration_project_id)
            where p.group_id = p_group_id
        ), '[]'::jsonb),
        'tasks', coalesce((
            select jsonb_agg(to_jsonb(t) order by t.created_at)
            from collaboration_project_task t
            join collaboration_project p using (collaboration_project_id)
            where p.group_id = p_group_id
        ), '[]'::jsonb),
        'updates', coalesce((
            select jsonb_agg(to_jsonb(u) order by u.created_at, u.collaboration_project_update_id)
            from collaboration_project_update u
            join collaboration_project p using (collaboration_project_id)
            where p.group_id = p_group_id
        ), '[]'::jsonb),
        'activities', coalesce((
            select jsonb_agg(to_jsonb(a) order by a.created_at desc, a.collaboration_project_activity_id desc)
            from collaboration_project_activity a
            join collaboration_project p using (collaboration_project_id)
            where p.group_id = p_group_id
        ), '[]'::jsonb),
        'outcomes', coalesce((
            select jsonb_agg(to_jsonb(o) order by o.created_at desc)
            from collaboration_project_outcome o
            join collaboration_project p using (collaboration_project_id)
            where p.group_id = p_group_id
        ), '[]'::jsonb),
        'sessions', coalesce((
            select jsonb_agg(
                to_jsonb(s) || jsonb_build_object(
                    'booked_count', (
                        select count(*)
                        from collaboration_office_hour_booking b
                        where b.collaboration_office_hour_session_id =
                            s.collaboration_office_hour_session_id
                          and b.status = 'booked'
                    ),
                    'available_capacity', greatest(
                        s.capacity - (
                            select count(*)
                            from collaboration_office_hour_booking b
                            where b.collaboration_office_hour_session_id =
                                s.collaboration_office_hour_session_id
                              and b.status = 'booked'
                        ),
                        0
                    ),
                    'can_book',
                        s.status = 'scheduled'
                        and s.starts_at > current_timestamp
                        and (
                            select count(*)
                            from collaboration_office_hour_booking b
                            where b.collaboration_office_hour_session_id =
                                s.collaboration_office_hour_session_id
                              and b.status = 'booked'
                        ) < s.capacity
                        and exists (
                            select 1
                            from collaboration_project_member m
                            where m.collaboration_project_id = s.collaboration_project_id
                              and m.user_id = p_actor_user_id
                              and m.invitation_status = 'accepted'
                        )
                )
                order by s.starts_at
            )
            from collaboration_office_hour_session s
            join collaboration_project p using (collaboration_project_id)
            where p.group_id = p_group_id
        ), '[]'::jsonb)
    );
$$ language sql stable;
