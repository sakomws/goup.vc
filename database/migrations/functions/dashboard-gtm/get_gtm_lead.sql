create or replace function get_gtm_lead(
    p_alliance_id uuid,
    p_gtm_lead_id uuid
) returns jsonb language plpgsql stable as $$
declare
    v_lead gtm_lead;
    v_activities jsonb;
    v_drafts jsonb;
    v_contacts jsonb;
    v_proposals jsonb;
    v_tasks jsonb;
    v_deliverables jsonb;
begin
    select * into v_lead
    from gtm_lead
    where gtm_lead_id = p_gtm_lead_id
      and alliance_id = p_alliance_id;

    if not found then
        return null;
    end if;

    select coalesce(jsonb_agg(jsonb_strip_nulls(jsonb_build_object(
        'gtm_lead_activity_id', a.gtm_lead_activity_id,
        'gtm_lead_id', a.gtm_lead_id,
        'actor_user_id', a.actor_user_id,
        'agent_id', a.agent_id,
        'kind', a.kind,
        'body', a.body,
        'details', a.details,
        'created_at', extract(epoch from a.created_at)::bigint
    )) order by a.created_at desc), '[]'::jsonb)
    into v_activities
    from gtm_lead_activity a
    where a.gtm_lead_id = p_gtm_lead_id;

    select coalesce(
        jsonb_agg(gtm_agent_draft_json(d) order by d.created_at desc),
        '[]'::jsonb
    )
    into v_drafts
    from gtm_agent_draft d
    where d.gtm_lead_id = p_gtm_lead_id;

    select coalesce(jsonb_agg(jsonb_build_object(
        'gtm_sponsor_contact_id', c.gtm_sponsor_contact_id,
        'user_id', c.user_id, 'name', c.name, 'email', c.email,
        'title', c.title, 'is_primary', c.is_primary
    ) order by c.is_primary desc, c.name), '[]'::jsonb)
    into v_contacts from gtm_sponsor_contact c where c.gtm_lead_id = p_gtm_lead_id;

    select coalesce(jsonb_agg(jsonb_build_object(
        'gtm_sponsor_proposal_id', p.gtm_sponsor_proposal_id,
        'gtm_sponsor_package_id', p.gtm_sponsor_package_id,
        'status', p.status, 'title', p.title, 'package_snapshot', p.package_snapshot,
        'amount_cents', p.amount_cents, 'currency', p.currency,
        'valid_until', p.valid_until, 'created_at', extract(epoch from p.created_at)::bigint
    ) order by p.created_at desc), '[]'::jsonb)
    into v_proposals from gtm_sponsor_proposal p where p.gtm_lead_id = p_gtm_lead_id;

    select coalesce(jsonb_agg(jsonb_build_object(
        'gtm_task_id', t.gtm_task_id, 'title', t.title, 'notes', t.notes,
        'due_at', extract(epoch from t.due_at)::bigint, 'state', t.state,
        'assigned_user_id', t.assigned_user_id
    ) order by (t.state = 'open') desc, t.due_at), '[]'::jsonb)
    into v_tasks from gtm_task t where t.gtm_lead_id = p_gtm_lead_id;

    select coalesce(jsonb_agg(jsonb_build_object(
        'gtm_sponsor_deliverable_id', d.gtm_sponsor_deliverable_id,
        'gtm_sponsor_proposal_id', d.gtm_sponsor_proposal_id,
        'title', d.title, 'notes', d.notes,
        'due_at', extract(epoch from d.due_at)::bigint, 'state', d.state
    ) order by (d.state not in ('delivered', 'waived')) desc, d.due_at), '[]'::jsonb)
    into v_deliverables from gtm_sponsor_deliverable d where d.gtm_lead_id = p_gtm_lead_id;

    return gtm_lead_json(v_lead)
        || jsonb_build_object(
            'activities', coalesce(v_activities, '[]'::jsonb),
            'drafts', coalesce(v_drafts, '[]'::jsonb),
            'contacts', coalesce(v_contacts, '[]'::jsonb),
            'proposals', coalesce(v_proposals, '[]'::jsonb),
            'tasks', coalesce(v_tasks, '[]'::jsonb),
            'deliverables', coalesce(v_deliverables, '[]'::jsonb)
        );
end;
$$;
