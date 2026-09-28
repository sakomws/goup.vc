create or replace function invite_collaboration_project_member(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_project_id uuid,
    p_user_id uuid,
    p_role text
)
returns uuid as $$
declare
    v_alliance_id uuid;
    v_invitation_attempt integer;
    v_member collaboration_project_member;
    v_member_id uuid;
begin
    if p_role not in ('contributor', 'viewer') then
        raise exception 'only contributor or viewer invitations are allowed';
    end if;

    select g.alliance_id
    into v_alliance_id
    from collaboration_project p
    join "group" g using (group_id)
    where p.collaboration_project_id = p_project_id
      and p.group_id = p_group_id
      and g.active = true
      and g.deleted = false;

    if v_alliance_id is null or not user_has_group_permission(
        v_alliance_id,
        p_group_id,
        p_actor_user_id,
        'group.projects.write'
    ) then
        raise exception 'project management permission required'
            using errcode = 'insufficient_privilege';
    end if;

    select *
    into v_member
    from collaboration_project_member
    where collaboration_project_id = p_project_id
      and user_id = p_user_id
    for update;

    if found then
        v_member_id := v_member.collaboration_project_member_id;

        if v_member.invitation_status = 'pending' and v_member.role = p_role then
            return v_member_id;
        end if;

        if v_member.role = 'owner' then
            raise exception 'project owner role cannot be changed by invitation';
        end if;

        if v_member.invitation_status = 'accepted' then
            if v_member.role <> p_role then
                update collaboration_project_member
                set role = p_role
                where collaboration_project_member_id = v_member_id;

                insert into collaboration_project_activity (
                    collaboration_project_id, actor_user_id, kind, subject_type,
                    subject_id, data
                ) values (
                    p_project_id, p_actor_user_id, 'member.role_changed', 'member',
                    v_member_id, jsonb_build_object('role', p_role)
                );
            end if;
            return v_member_id;
        end if;

        v_invitation_attempt := v_member.invitation_attempt + 1;
        update collaboration_project_member
        set role = p_role,
            invitation_status = 'pending',
            invited_by = p_actor_user_id,
            invited_at = current_timestamp,
            responded_at = null,
            invitation_attempt = v_invitation_attempt
        where collaboration_project_member_id = v_member_id;
    else
        insert into collaboration_project_member (
            collaboration_project_id, user_id, role, invitation_status, invited_by
        ) values (
            p_project_id, p_user_id, p_role, 'pending', p_actor_user_id
        )
        returning collaboration_project_member_id, invitation_attempt
        into v_member_id, v_invitation_attempt;
    end if;

    insert into collaboration_notification_delivery (
        collaboration_project_id, recipient_user_id, kind, idempotency_key, payload
    ) values (
        p_project_id, p_user_id, 'invitation',
        'invitation:' || v_member_id::text || ':' || v_invitation_attempt::text,
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
