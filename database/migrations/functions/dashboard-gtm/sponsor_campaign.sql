create or replace function list_gtm_sponsor_packages(p_alliance_id uuid, p_group_id uuid)
returns jsonb language sql stable as $$
    select coalesce(jsonb_agg(jsonb_build_object(
        'gtm_sponsor_package_id', p.gtm_sponsor_package_id,
        'alliance_id', p.alliance_id,
        'group_id', p.group_id,
        'name', p.name,
        'description', p.description,
        'price_cents', p.price_cents,
        'currency', p.currency,
        'billing_period', p.billing_period,
        'active', p.active,
        'deliverables', coalesce((
            select jsonb_agg(jsonb_build_object(
                'name', d.name, 'description', d.description, 'quantity', d.quantity
            ) order by d.sort_order, d.gtm_sponsor_package_deliverable_id)
            from gtm_sponsor_package_deliverable d
            where d.gtm_sponsor_package_id = p.gtm_sponsor_package_id
        ), '[]'::jsonb)
    ) order by p.active desc, p.name), '[]'::jsonb)
    from gtm_sponsor_package p
    where p.alliance_id = p_alliance_id
      and (p.group_id is null or p.group_id = p_group_id);
$$;

create or replace function add_gtm_sponsor_package(
    p_actor_user_id uuid, p_alliance_id uuid, p_input jsonb
) returns uuid language plpgsql as $$
declare
    v_id uuid;
    v_group_id uuid := nullif(p_input->>'group_id', '')::uuid;
    v_item jsonb;
    v_position integer := 0;
begin
    if v_group_id is not null and not exists (
        select 1 from "group" where group_id = v_group_id
          and alliance_id = p_alliance_id and deleted = false
    ) then
        raise exception 'group does not belong to alliance';
    end if;

    insert into gtm_sponsor_package (
        alliance_id, group_id, name, description, price_cents, currency, billing_period
    ) values (
        p_alliance_id, v_group_id, trim(p_input->>'name'),
        nullif(trim(p_input->>'description'), ''),
        (p_input->>'price_cents')::bigint, upper(trim(p_input->>'currency')),
        coalesce(nullif(trim(p_input->>'billing_period'), ''), 'one_time')
    ) returning gtm_sponsor_package_id into v_id;

    for v_item in select value from jsonb_array_elements(coalesce(p_input->'deliverables', '[]'::jsonb))
    loop
        insert into gtm_sponsor_package_deliverable (
            gtm_sponsor_package_id, name, description, quantity, sort_order
        ) values (
            v_id, trim(v_item->>'name'), nullif(trim(v_item->>'description'), ''),
            coalesce((v_item->>'quantity')::integer, 1), v_position
        );
        v_position := v_position + 1;
    end loop;

    perform insert_audit_log(
        'gtm_sponsor_package_created', p_actor_user_id, 'gtm_sponsor_package',
        v_id, p_alliance_id, v_group_id
    );
    return v_id;
end;
$$;

create or replace function add_gtm_sponsor_contact(
    p_actor_user_id uuid, p_alliance_id uuid, p_gtm_lead_id uuid, p_input jsonb
) returns uuid language plpgsql as $$
declare
    v_id uuid;
begin
    if not exists (
        select 1 from gtm_lead where gtm_lead_id = p_gtm_lead_id
          and alliance_id = p_alliance_id and kind = 'sponsor'
    ) then raise exception 'sponsor lead not found'; end if;

    if coalesce((p_input->>'is_primary')::boolean, false) then
        update gtm_sponsor_contact set is_primary = false
        where gtm_lead_id = p_gtm_lead_id and is_primary;
    end if;

    insert into gtm_sponsor_contact (gtm_lead_id, user_id, name, email, title, is_primary)
    values (
        p_gtm_lead_id, nullif(p_input->>'user_id', '')::uuid, trim(p_input->>'name'),
        nullif(trim(p_input->>'email'), ''), nullif(trim(p_input->>'title'), ''),
        coalesce((p_input->>'is_primary')::boolean, false)
    ) returning gtm_sponsor_contact_id into v_id;

    perform add_gtm_lead_activity(
        p_actor_user_id, p_gtm_lead_id, null, 'note',
        format('Added sponsor contact %s', trim(p_input->>'name')),
        jsonb_build_object('sponsor_contact_id', v_id)
    );
    return v_id;
end;
$$;

create or replace function add_manual_gtm_lead_activity(
    p_actor_user_id uuid, p_alliance_id uuid, p_gtm_lead_id uuid, p_kind text, p_body text
) returns uuid language plpgsql as $$
begin
    if not exists (
        select 1 from gtm_lead where gtm_lead_id = p_gtm_lead_id and alliance_id = p_alliance_id
    ) then raise exception 'gtm lead not found'; end if;
    return add_gtm_lead_activity(
        p_actor_user_id, p_gtm_lead_id, null, p_kind, trim(p_body),
        jsonb_build_object('manual', true)
    );
end;
$$;

create or replace function add_gtm_sponsor_proposal(
    p_actor_user_id uuid, p_alliance_id uuid, p_gtm_lead_id uuid, p_input jsonb
) returns uuid language plpgsql as $$
declare
    v_id uuid;
    v_package gtm_sponsor_package;
    v_snapshot jsonb;
begin
    if not exists (
        select 1 from gtm_lead where gtm_lead_id = p_gtm_lead_id
          and alliance_id = p_alliance_id and kind = 'sponsor'
    ) then raise exception 'sponsor lead not found'; end if;

    select p.* into v_package
    from gtm_sponsor_package p
    join gtm_lead l on l.gtm_lead_id = p_gtm_lead_id
    where p.gtm_sponsor_package_id = (p_input->>'gtm_sponsor_package_id')::uuid
      and p.alliance_id = p_alliance_id and p.active
      and (p.group_id is null or p.group_id = l.group_id);
    if not found then raise exception 'sponsor package not found'; end if;

    v_snapshot := jsonb_build_object(
        'name', v_package.name,
        'description', v_package.description,
        'price_cents', v_package.price_cents,
        'currency', v_package.currency,
        'billing_period', v_package.billing_period,
        'deliverables', coalesce((
            select jsonb_agg(jsonb_build_object(
                'name', d.name, 'description', d.description, 'quantity', d.quantity
            ) order by d.sort_order, d.gtm_sponsor_package_deliverable_id)
            from gtm_sponsor_package_deliverable d
            where d.gtm_sponsor_package_id = v_package.gtm_sponsor_package_id
        ), '[]'::jsonb)
    );

    insert into gtm_sponsor_proposal (
        gtm_lead_id, gtm_sponsor_package_id, created_by, status, title,
        package_snapshot, amount_cents, currency, valid_until
    ) values (
        p_gtm_lead_id, v_package.gtm_sponsor_package_id, p_actor_user_id, 'draft',
        trim(p_input->>'title'), v_snapshot,
        coalesce(nullif(p_input->>'amount_cents', '')::bigint, v_package.price_cents),
        coalesce(upper(nullif(trim(p_input->>'currency'), '')), v_package.currency),
        nullif(p_input->>'valid_until', '')::date
    ) returning gtm_sponsor_proposal_id into v_id;

    perform add_gtm_lead_activity(
        p_actor_user_id, p_gtm_lead_id, null, 'note',
        format('Created proposal %s', trim(p_input->>'title')),
        jsonb_build_object('sponsor_proposal_id', v_id)
    );
    return v_id;
end;
$$;

create or replace function add_gtm_task(
    p_actor_user_id uuid, p_alliance_id uuid, p_gtm_lead_id uuid, p_input jsonb
) returns uuid language plpgsql as $$
declare v_id uuid;
begin
    if not exists (
        select 1 from gtm_lead where gtm_lead_id = p_gtm_lead_id and alliance_id = p_alliance_id
    ) then raise exception 'gtm lead not found'; end if;
    insert into gtm_task (
        gtm_lead_id, assigned_user_id, created_by, title, notes, due_at, reminder_key
    ) values (
        p_gtm_lead_id, nullif(p_input->>'assigned_user_id', '')::uuid, p_actor_user_id,
        trim(p_input->>'title'), nullif(trim(p_input->>'notes'), ''),
        (p_input->>'due_at')::timestamptz, nullif(trim(p_input->>'reminder_key'), '')
    )
    on conflict (gtm_lead_id, reminder_key) where reminder_key is not null
    do update set updated_at = gtm_task.updated_at
    returning gtm_task_id into v_id;
    return v_id;
end;
$$;

create or replace function set_gtm_sponsor_proposal_status(
    p_actor_user_id uuid, p_alliance_id uuid, p_group_id uuid, p_id uuid, p_status text
) returns void language plpgsql as $$
declare v_lead_id uuid;
begin
    if p_status not in ('draft', 'sent', 'accepted', 'declined', 'expired', 'withdrawn') then
        raise exception 'invalid proposal status';
    end if;
    update gtm_sponsor_proposal p set status = p_status,
        sent_at = case
            when p_status = 'sent' and sent_at is null then current_timestamp else sent_at
        end,
        accepted_at = case
            when p_status = 'accepted' then current_timestamp
            when p_status = 'draft' then null else accepted_at
        end,
        updated_at = current_timestamp
    from gtm_lead l
    where p.gtm_sponsor_proposal_id = p_id and l.gtm_lead_id = p.gtm_lead_id
      and l.alliance_id = p_alliance_id
      and (p_group_id is null or l.group_id = p_group_id)
    returning p.gtm_lead_id into v_lead_id;
    if not found then raise exception 'sponsor proposal not found'; end if;
    perform add_gtm_lead_activity(
        p_actor_user_id, v_lead_id, null, 'note',
        format('Proposal marked %s', p_status),
        jsonb_build_object('gtm_sponsor_proposal_id', p_id)
    );
end;
$$;

create or replace function set_gtm_task_state(
    p_actor_user_id uuid, p_alliance_id uuid, p_group_id uuid, p_gtm_task_id uuid, p_state text
) returns void language plpgsql as $$
declare v_lead_id uuid;
begin
    if p_state not in ('open', 'completed', 'cancelled') then
        raise exception 'invalid task state';
    end if;
    update gtm_task t set state = p_state,
        completed_at = case when p_state = 'completed' then current_timestamp else null end,
        updated_at = current_timestamp
    from gtm_lead l
    where t.gtm_task_id = p_gtm_task_id and l.gtm_lead_id = t.gtm_lead_id
      and l.alliance_id = p_alliance_id
      and (p_group_id is null or l.group_id = p_group_id)
    returning t.gtm_lead_id into v_lead_id;
    if not found then raise exception 'gtm task not found'; end if;
    perform add_gtm_lead_activity(
        p_actor_user_id, v_lead_id, null, 'note',
        format('Task marked %s', p_state), jsonb_build_object('gtm_task_id', p_gtm_task_id)
    );
end;
$$;

create or replace function list_due_gtm_tasks(
    p_alliance_id uuid, p_group_id uuid, p_due_before timestamptz default current_timestamp
) returns jsonb language sql stable as $$
    select coalesce(jsonb_agg(jsonb_build_object(
        'gtm_task_id', t.gtm_task_id, 'gtm_lead_id', t.gtm_lead_id,
        'lead_name', l.name, 'title', t.title, 'notes', t.notes,
        'due_at', extract(epoch from t.due_at)::bigint, 'state', t.state,
        'assigned_user_id', t.assigned_user_id, 'reminder_key', t.reminder_key
    ) order by t.due_at, t.gtm_task_id), '[]'::jsonb)
    from gtm_task t join gtm_lead l using (gtm_lead_id)
    where l.alliance_id = p_alliance_id
      and (p_group_id is null or l.group_id = p_group_id)
      and t.state = 'open' and t.due_at <= p_due_before;
$$;

create or replace function add_gtm_sponsor_deliverable(
    p_actor_user_id uuid, p_alliance_id uuid, p_gtm_lead_id uuid, p_input jsonb
) returns uuid language plpgsql as $$
declare
    v_id uuid;
    v_proposal_id uuid := nullif(p_input->>'gtm_sponsor_proposal_id', '')::uuid;
begin
    if not exists (
        select 1 from gtm_lead where gtm_lead_id = p_gtm_lead_id
          and alliance_id = p_alliance_id and kind = 'sponsor'
    ) then raise exception 'sponsor lead not found'; end if;
    if v_proposal_id is not null and not exists (
        select 1 from gtm_sponsor_proposal
        where gtm_sponsor_proposal_id = v_proposal_id
          and gtm_lead_id = p_gtm_lead_id
    ) then raise exception 'sponsor proposal does not belong to lead'; end if;
    insert into gtm_sponsor_deliverable (
        gtm_lead_id, gtm_sponsor_proposal_id, title, notes, due_at
    ) values (
        p_gtm_lead_id, v_proposal_id,
        trim(p_input->>'title'), nullif(trim(p_input->>'notes'), ''),
        nullif(p_input->>'due_at', '')::timestamptz
    ) returning gtm_sponsor_deliverable_id into v_id;
    return v_id;
end;
$$;

create or replace function set_gtm_sponsor_deliverable_state(
    p_actor_user_id uuid, p_alliance_id uuid, p_group_id uuid, p_id uuid, p_state text
) returns void language plpgsql as $$
declare v_lead_id uuid;
begin
    if p_state not in ('pending', 'in_progress', 'delivered', 'waived') then
        raise exception 'invalid deliverable state';
    end if;
    update gtm_sponsor_deliverable d set state = p_state,
        completed_at = case when p_state = 'delivered' then current_timestamp else null end,
        updated_at = current_timestamp
    from gtm_lead l
    where d.gtm_sponsor_deliverable_id = p_id and l.gtm_lead_id = d.gtm_lead_id
      and l.alliance_id = p_alliance_id
      and (p_group_id is null or l.group_id = p_group_id)
    returning d.gtm_lead_id into v_lead_id;
    if not found then raise exception 'sponsor deliverable not found'; end if;
    perform add_gtm_lead_activity(
        p_actor_user_id, v_lead_id, null, 'note',
        format('Deliverable marked %s', p_state),
        jsonb_build_object('gtm_sponsor_deliverable_id', p_id)
    );
end;
$$;
