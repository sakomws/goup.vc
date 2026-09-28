begin;
select plan(3);

\set allianceID 'c1160000-0000-0000-0000-000000000001'
\set groupCategoryID 'c1160000-0000-0000-0000-000000000002'
\set groupID 'c1160000-0000-0000-0000-000000000003'

insert into alliance (
    alliance_id,
    name,
    display_name,
    description,
    banner_mobile_url,
    banner_url,
    logo_url
) values (
    :'allianceID',
    'gtm-draft-list',
    'GTM Draft List',
    'Alliance used for GTM draft list tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Tech');

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values (:'groupID', :'allianceID', :'groupCategoryID', 'GTM Group', 'gtm-draft-group');

select add_gtm_agent_draft(
    null::uuid,
    :'allianceID'::uuid,
    jsonb_build_object(
        'agent_id', 'lead_generation',
        'title', 'Alliance suggestions',
        'body', 'Review alliance suggestions',
        'suggested_stage', 'lead_generation',
        'payload', jsonb_build_object('candidates', '[]'::jsonb)
    )
) as "allianceDraftID" \gset

select add_gtm_agent_draft(
    null::uuid,
    :'allianceID'::uuid,
    jsonb_build_object(
        'group_id', :'groupID',
        'agent_id', 'lead_generation',
        'title', 'Group suggestions',
        'body', 'Review group suggestions',
        'suggested_stage', 'lead_generation',
        'payload', jsonb_build_object('candidates', '[]'::jsonb)
    )
) as "groupDraftID" \gset

select is(
    list_gtm_agent_drafts(
        :'allianceID'::uuid,
        jsonb_build_object(
            'agent_id', 'lead_generation',
            'status', 'pending',
            'exact_scope', true
        )
    )#>>'{drafts,0,gtm_agent_draft_id}',
    :'allianceDraftID',
    'Exact alliance scope excludes group drafts'
);

select is(
    list_gtm_agent_drafts(
        :'allianceID'::uuid,
        jsonb_build_object(
            'agent_id', 'lead_generation',
            'status', 'pending',
            'group_id', :'groupID',
            'exact_scope', true
        )
    )#>>'{drafts,0,gtm_agent_draft_id}',
    :'groupDraftID',
    'Exact group scope returns its lead-generation draft'
);

select review_gtm_agent_draft(
    null::uuid,
    :'allianceID'::uuid,
    :'groupDraftID'::uuid,
    jsonb_build_object('status', 'rejected')
);

select is(
    jsonb_array_length(
        list_gtm_agent_drafts(
            :'allianceID'::uuid,
            jsonb_build_object(
                'agent_id', 'lead_generation',
                'status', 'pending',
                'group_id', :'groupID',
                'exact_scope', true
            )
        )->'drafts'
    ),
    0,
    'Rejected drafts leave the pending group review list'
);

select * from finish();
rollback;
