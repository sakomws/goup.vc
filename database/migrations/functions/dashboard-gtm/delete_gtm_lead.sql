create or replace function delete_gtm_lead(
    p_actor_user_id uuid,
    p_alliance_id uuid,
    p_gtm_lead_id uuid,
    p_group_id uuid default null
) returns void language plpgsql as $$
declare
    v_lead gtm_lead;
begin
    select * into v_lead
    from gtm_lead
    where gtm_lead_id = p_gtm_lead_id
      and alliance_id = p_alliance_id;

    if not found then
        raise exception 'gtm lead not found';
    end if;

    if p_group_id is not null and v_lead.group_id is distinct from p_group_id then
        raise exception 'gtm lead not found';
    end if;

    perform insert_audit_log(
        'gtm_lead_deleted',
        p_actor_user_id,
        'gtm_lead',
        p_gtm_lead_id,
        p_alliance_id,
        v_lead.group_id
    );

    delete from gtm_lead
    where gtm_lead_id = p_gtm_lead_id
      and alliance_id = p_alliance_id;
end;
$$;
