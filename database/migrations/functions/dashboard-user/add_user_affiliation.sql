-- add_user_affiliation links the user to a landscape entry with a role,
-- updating the role if the affiliation already exists.
create or replace function add_user_affiliation(
    p_user_id uuid,
    p_landscape_entry_id uuid,
    p_role text
)
returns void as $$
declare
    v_user_affiliation_id uuid;
begin
    if p_role not in (
        'founder',
        'co_founder',
        'executive',
        'maintainer',
        'contributor',
        'representative',
        'other'
    ) then
        raise exception 'invalid affiliation role: %', p_role
            using errcode = 'check_violation';
    end if;

    if not exists (
        select 1
        from landscape_entry
        where landscape_entry_id = p_landscape_entry_id
          and published = true
    ) then
        raise exception 'landscape entry not found'
            using errcode = 'no_data_found';
    end if;

    insert into user_affiliation (user_id, landscape_entry_id, role)
    values (p_user_id, p_landscape_entry_id, p_role)
    on conflict (user_id, landscape_entry_id) do update set
        role = excluded.role
    returning user_affiliation_id into v_user_affiliation_id;

    perform insert_audit_log(
        'user_affiliation_added',
        p_user_id,
        'user_affiliation',
        v_user_affiliation_id,
        p_details => jsonb_build_object(
            'landscape_entry_id', p_landscape_entry_id,
            'role', p_role
        )
    );
end;
$$ language plpgsql;
