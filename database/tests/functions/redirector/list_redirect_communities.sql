-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

<<<<<<<< HEAD:database/tests/functions/common/list_redirect_alliances.sql
\set activeAllianceID '0c130000-0000-0000-0000-000000000001'
\set fallbackAllianceID '0c130000-0000-0000-0000-000000000002'
\set inactiveAllianceID '0c130000-0000-0000-0000-000000000003'
========
\set activeCommunityID '0c000000-0000-0000-0000-000000000001'
\set fallbackCommunityID '0c000000-0000-0000-0000-000000000002'
\set inactiveCommunityID '0c000000-0000-0000-0000-000000000003'
>>>>>>>> 14b0d0c1 (Database layer hardening (#675)):database/tests/functions/redirector/list_redirect_communities.sql

-- ============================================================================
-- SEED DATA
-- ============================================================================

<<<<<<<< HEAD:database/tests/functions/common/list_redirect_alliances.sql
-- Alliances
insert into alliance (
    alliance_id,
    name,
    display_name,
    description,
    banner_mobile_url,
    banner_url,
    logo_url
) values (
    :'activeAllianceID',
    'active-alliance',
    'Active Alliance',
    'An active alliance',
    'https://example.com/banner-mobile-active.png',
    'https://example.com/banner-active.png',
    'https://example.com/logo-active.png'
), (
    :'fallbackAllianceID',
    'fallback-alliance',
    'Fallback Alliance',
    'A alliance with a fallback URL',
    'https://example.com/banner-mobile-fallback.png',
    'https://example.com/banner-fallback.png',
    'https://example.com/logo-fallback.png'
), (
    :'inactiveAllianceID',
    'inactive-alliance',
    'Inactive Alliance',
    'A disabled alliance',
    'https://example.com/banner-mobile-inactive.png',
    'https://example.com/banner-inactive.png',
    'https://example.com/logo-inactive.png'
);
========
-- Communities
select fx_community(:'activeCommunityID', jsonb_build_object('name', 'active-community-list-redirect-communities'));
select fx_community(:'fallbackCommunityID', jsonb_build_object('name', 'fallback-community'));
select fx_community(:'inactiveCommunityID', jsonb_build_object(
    'active', false,
    'name', 'inactive-community-list-redirect-communities'
));
>>>>>>>> 14b0d0c1 (Database layer hardening (#675)):database/tests/functions/redirector/list_redirect_communities.sql

update alliance
set active = false
where alliance_id = :'inactiveAllianceID'::uuid;

-- Redirect settings
insert into alliance_redirect_settings (
    alliance_id,

    base_legacy_url
) values
    (:'fallbackAllianceID', 'https://legacy.example.org'),
    (:'inactiveAllianceID', 'https://inactive.example.org');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return active alliances ordered by alliance name
select is(
    (
        select jsonb_agg(row_to_json(r))
        from list_redirect_alliances() r
    ),
    '[
<<<<<<<< HEAD:database/tests/functions/common/list_redirect_alliances.sql
        {"alliance_name": "active-alliance", "base_legacy_url": null},
        {"alliance_name": "fallback-alliance", "base_legacy_url": "https://legacy.example.org"}
========
        {"community_name": "active-community-list-redirect-communities", "base_legacy_url": null},
        {"community_name": "fallback-community", "base_legacy_url": "https://legacy.example.org"}
>>>>>>>> 14b0d0c1 (Database layer hardening (#675)):database/tests/functions/redirector/list_redirect_communities.sql
    ]'::jsonb,
    'Should return active alliances ordered by alliance name'
);

-- Should exclude inactive alliances
select ok(
    not exists(
        select 1
<<<<<<<< HEAD:database/tests/functions/common/list_redirect_alliances.sql
        from list_redirect_alliances()
        where alliance_name = 'inactive-alliance'
========
        from list_redirect_communities()
        where community_name = 'inactive-community-list-redirect-communities'
>>>>>>>> 14b0d0c1 (Database layer hardening (#675)):database/tests/functions/redirector/list_redirect_communities.sql
    ),
    'Should exclude inactive alliances'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
