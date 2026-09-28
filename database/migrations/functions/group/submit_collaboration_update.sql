create or replace function submit_collaboration_update(
    p_user_id uuid,
    p_project_id uuid,
    p_body text,
    p_blockers text,
    p_next_steps text
)
returns uuid as $$
declare
    v_member_id uuid;
    v_update_id uuid;
begin
    select m.collaboration_project_member_id into v_member_id
    from collaboration_project_member m
    join collaboration_project p using (collaboration_project_id)
    where m.collaboration_project_id = p_project_id
      and m.user_id = p_user_id
      and m.invitation_status = 'accepted'
      and m.role in ('owner', 'contributor')
      and p.lifecycle not in ('completed', 'archived');

    if v_member_id is null then
        raise exception 'accepted owner or contributor membership required'
            using errcode = 'insufficient_privilege';
    end if;

    insert into collaboration_project_update (
        collaboration_project_id, collaboration_project_member_id,
        body, blockers, next_steps
    ) values (
        p_project_id, v_member_id, btrim(p_body),
        nullif(btrim(p_blockers), ''), nullif(btrim(p_next_steps), '')
    ) returning collaboration_project_update_id into v_update_id;

    insert into collaboration_project_activity (
        collaboration_project_id, actor_user_id, kind, subject_type, subject_id
    ) values (p_project_id, p_user_id, 'update.posted', 'update', v_update_id);

    return v_update_id;
end;
$$ language plpgsql;
