-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(5);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID 'c1150000-0000-0000-0000-000000000001'
\set groupCategoryID 'c1150000-0000-0000-0000-000000000002'
\set groupID 'c1150000-0000-0000-0000-000000000003'
\set otherGroupID 'c1150000-0000-0000-0000-000000000004'
\set missingLeadID 'c1150000-0000-0000-0000-000000000099'

-- ============================================================================
-- SEED DATA
-- ============================================================================

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
    'gtm-delete-alliance',
    'GTM Delete Alliance',
    'Alliance used for GTM delete tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Tech');

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values
    (:'groupID', :'allianceID', :'groupCategoryID', 'GTM Group', 'gtm-delete-group'),
    (:'otherGroupID', :'allianceID', :'groupCategoryID', 'Other Group', 'gtm-delete-other');

create temporary table gtm_delete_fixture (
    gtm_lead_id uuid primary key
);

insert into gtm_delete_fixture (gtm_lead_id)
select add_gtm_lead(null::uuid, :'allianceID'::uuid, jsonb_build_object(
    'name', 'Delete Me',
    'kind', 'sponsor',
    'email', 'delete-me@example.com',
    'group_id', :'groupID',
    'source', 'manual'
));

-- ============================================================================
-- TESTS
-- ============================================================================

select lives_ok(
    format(
        $sql$
        select delete_gtm_lead(null::uuid, %L::uuid, (select gtm_lead_id from gtm_delete_fixture))
        $sql$,
        :'allianceID'
    ),
    'Should delete a GTM lead'
);

select is(
    (
        select count(*)::int
        from gtm_lead
        where gtm_lead_id = (select gtm_lead_id from gtm_delete_fixture)
    ),
    0,
    'Should remove the lead row'
);

select is(
    (
        select count(*)::int
        from gtm_lead_activity
        where gtm_lead_id = (select gtm_lead_id from gtm_delete_fixture)
    ),
    0,
    'Should cascade activity rows with the lead'
);

select throws_ok(
    format(
        $sql$
        select delete_gtm_lead(null::uuid, %L::uuid, %L::uuid)
        $sql$,
        :'allianceID',
        :'missingLeadID'
    ),
    'P0001',
    'gtm lead not found',
    'Should reject deleting a missing lead'
);

select throws_ok(
    format(
        $sql$
        do $body$
        declare
            v_lead_id uuid;
        begin
            v_lead_id := add_gtm_lead(null::uuid, %L::uuid, jsonb_build_object(
                'name', 'Wrong Group',
                'kind', 'startup',
                'email', 'wrong-group@example.com',
                'group_id', %L,
                'source', 'manual'
            ));
            perform delete_gtm_lead(null::uuid, %L::uuid, v_lead_id, %L::uuid);
        end;
        $body$;
        $sql$,
        :'allianceID',
        :'groupID',
        :'allianceID',
        :'otherGroupID'
    ),
    'P0001',
    'gtm lead not found',
    'Should reject deleting a lead from another group'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
