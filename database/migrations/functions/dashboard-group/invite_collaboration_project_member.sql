create or replace function invite_collaboration_project_member(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_project_id uuid,
    p_user_id uuid,
    p_role text
)
returns uuid as $$
declare
    v_member_id uuid;
begin
    if p_role not in ('contributor', 'viewer') then
        raise exception 'only contributor or viewer invitations are allowed';
    end if;

    insert into collaboration_project_member (
        collaboration_project_id, user_id, role, invitation_status, invited_by
    )
    select p_project_id, p_user_id, p_role, 'pending', p_actor_user_id
    from collaboration_project p
    where p.collaboration_project_id = p_project_id and p.group_id = p_group_id
    on conflict (collaboration_project_id, user_id) do update
    set role = excluded.role,
        invitation_status = case
            when collaboration_project_member.invitation_status = 'accepted'
                then 'accepted'
            else 'pending'
        end,
        invited_by = excluded.invited_by,
        invited_at = current_timestamp
    returning collaboration_project_member_id into v_member_id;

    if v_member_id is null then
        raise exception 'project not found in group';
    end if;

    insert into collaboration_notification_delivery (
        collaboration_project_id, recipient_user_id, kind, idempotency_key, payload
    ) values (
        p_project_id, p_user_id, 'invitation',
        'invitation:' || v_member_id::text,
        jsonb_build_object('membership_id', v_member_id, 'role', p_role)
    ) on conflict do nothing;

    insert into collaboration_project_activity (
        collaboration_project_id, actor_user_id, kind, subject_type, subject_id,
        data
    ) values (
        p_project_id, p_actor_user_id, 'member.invited', 'member', v_member_id,
        jsonb_build_object('role', p_role)
    );

    return v_member_id;
end;
$$ language plpgsql;
