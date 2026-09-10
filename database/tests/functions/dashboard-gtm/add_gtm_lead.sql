-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID 'a1150000-0000-0000-0000-000000000001'
\set groupCategoryID 'a1150000-0000-0000-0000-000000000002'
\set groupID 'a1150000-0000-0000-0000-000000000003'

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
    'gtm-alliance',
    'GTM Alliance',
    'Alliance used for GTM tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Tech');

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values (:'groupID', :'allianceID', :'groupCategoryID', 'GTM Group', 'gtm-group');

-- ============================================================================
-- TESTS
-- ============================================================================

select is(
    (
        select (get_gtm_lead(
            :'allianceID'::uuid,
            add_gtm_lead(null::uuid, :'allianceID'::uuid, jsonb_build_object(
                'name', 'Ada Example',
                'kind', 'sponsor',
                'org_name', 'Example Co',
                'email', 'ada@example.com',
                'website_url', 'https://example.com',
                'group_id', :'groupID',
                'source', 'manual'
            ))
        )::jsonb)->>'name'
    ),
    'Ada Example',
    'Should create a GTM lead and return it'
);

select is(
    (
        select count(*)::int
        from gtm_lead_activity
        where gtm_lead_id in (select gtm_lead_id from gtm_lead where email = 'ada@example.com')
    ),
    1,
    'Should write a creation activity row'
);

select throws_ok(
    format(
        $sql$
        select add_gtm_lead(null::uuid, %L::uuid, jsonb_build_object(
            'name', 'Duplicate',
            'kind', 'startup',
            'email', 'ada@example.com'
        ))
        $sql$,
        :'allianceID'
    ),
    'P0001',
    'gtm lead already exists for this email or website',
    'Should reject duplicate emails in the same alliance'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
