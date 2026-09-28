begin;
select plan(9);

\set allianceID 'c1190000-0000-0000-0000-000000000001'

insert into alliance (
    alliance_id, name, display_name, description, banner_mobile_url, banner_url, logo_url
) values (
    :'allianceID', 'sponsor-growth', 'Sponsor Growth', 'Sponsor campaign tests',
    'https://example.com/mobile.png', 'https://example.com/banner.png', 'https://example.com/logo.png'
);

select add_gtm_lead(null, :'allianceID', jsonb_build_object(
    'name', 'Sponsor Inc', 'kind', 'sponsor', 'email', 'partner@example.com'
)) as "leadID" \gset

select add_gtm_sponsor_package(null, :'allianceID', jsonb_build_object(
    'name', 'Gold', 'price_cents', 500000, 'currency', 'USD',
    'billing_period', 'annual',
    'deliverables', jsonb_build_array(jsonb_build_object('name', 'Logo placement', 'quantity', 1))
)) as "packageID" \gset

select is(
    jsonb_array_length(list_gtm_sponsor_packages(:'allianceID', null)),
    1,
    'lists normalized sponsor packages'
);

select add_gtm_sponsor_contact(null, :'allianceID', :'leadID', jsonb_build_object(
    'name', 'Pat Partner', 'email', 'pat@example.com', 'is_primary', true
)) as "contactID" \gset

select is(
    (get_gtm_lead(:'allianceID', :'leadID')->'contacts'->0->>'name'),
    'Pat Partner',
    'returns linked sponsor contacts'
);

select add_gtm_sponsor_proposal(null, :'allianceID', :'leadID', jsonb_build_object(
    'gtm_sponsor_package_id', :'packageID', 'title', 'Annual partnership'
)) as "proposalID" \gset

update gtm_sponsor_package set price_cents = 700000
where gtm_sponsor_package_id = :'packageID';

select is(
    (select package_snapshot->>'price_cents' from gtm_sponsor_proposal
     where gtm_sponsor_proposal_id = :'proposalID'),
    '500000',
    'proposal package terms are immutable snapshots'
);

select add_gtm_task(null, :'allianceID', :'leadID', jsonb_build_object(
    'title', 'Follow up', 'due_at', current_timestamp - interval '1 hour',
    'reminder_key', 'sponsor-growth-follow-up'
)) as "taskID" \gset

select is(
    add_gtm_task(null, :'allianceID', :'leadID', jsonb_build_object(
        'title', 'Follow up duplicate', 'due_at', current_timestamp,
        'reminder_key', 'sponsor-growth-follow-up'
    )),
    :'taskID'::uuid,
    'task creation is idempotent by reminder key'
);

select is(
    jsonb_array_length(list_due_gtm_tasks(:'allianceID', null)),
    1,
    'due queue returns open due tasks'
);

select lives_ok(
    format('select set_gtm_task_state(null, %L, null, %L, %L)', :'allianceID', :'taskID', 'completed'),
    'tasks can be completed'
);

select is(
    jsonb_array_length(list_due_gtm_tasks(:'allianceID', null)),
    0,
    'completed tasks leave the due queue'
);

select add_gtm_lead(null, :'allianceID', jsonb_build_object(
    'name', 'Other Sponsor', 'kind', 'sponsor', 'email', 'other@example.com'
)) as "otherLeadID" \gset

select throws_ok(
    format(
        'select add_gtm_sponsor_deliverable(null, %L, %L, %L::jsonb)',
        :'allianceID',
        :'otherLeadID',
        jsonb_build_object(
            'gtm_sponsor_proposal_id', :'proposalID',
            'title', 'Wrong lead deliverable'
        )
    ),
    'P0001',
    'sponsor proposal does not belong to lead',
    'deliverables cannot reference another lead''s proposal'
);

select lives_ok(
    format(
        'select transition_gtm_lead(null, %L, %L, %L, true, %L::jsonb)',
        :'allianceID', :'leadID', 'lost', '{"lost_reason":"budget"}'
    ),
    'standard lost reasons remain human-approved transitions'
);

select * from finish();
rollback;
