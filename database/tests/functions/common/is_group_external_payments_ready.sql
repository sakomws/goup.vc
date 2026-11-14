-- Tests group-level external-payments readiness.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set communityID 'e07e0000-0000-0000-0000-000000000001'
\set groupCategoryID 'e07e0000-0000-0000-0000-000000000002'
\set groupDisabledID 'e07e0000-0000-0000-0000-000000000003'
\set groupReadyID 'e07e0000-0000-0000-0000-000000000004'
\set groupUnlistedID 'e07e0000-0000-0000-0000-000000000005'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Operator allowlist used by the group-readiness scenarios
insert into external_payments_config (
    allowed_countries,
    default_payment_window_hours,
    max_payment_window_hours
) values (
    array['KR']::text[],
    72,
    336
);

select fx_community(:'communityID');
select fx_group_category(:'groupCategoryID', :'communityID');
select fx_group(
    :'groupReadyID', :'communityID', :'groupCategoryID',
    jsonb_build_object('country_code', 'KR', 'external_payments_enabled', true)
);
select fx_group(
    :'groupDisabledID', :'communityID', :'groupCategoryID',
    jsonb_build_object('country_code', 'KR', 'external_payments_enabled', false)
);
select fx_group(
    :'groupUnlistedID', :'communityID', :'groupCategoryID',
    jsonb_build_object('country_code', 'US', 'external_payments_enabled', true)
);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should report ready when the toggle is on and the country is allowlisted
select is(
    is_group_external_payments_ready(:'groupReadyID'::uuid),
    true,
    'Should report ready when the toggle is on and the country is allowlisted'
);

-- Should report unready when the group toggle is off
select is(
    is_group_external_payments_ready(:'groupDisabledID'::uuid),
    false,
    'Should report unready when the group toggle is off'
);

-- Should report unready when the group country is not allowlisted
select is(
    is_group_external_payments_ready(:'groupUnlistedID'::uuid),
    false,
    'Should report unready when the group country is not allowlisted'
);

-- Should report unready for an unknown group
select is(
    is_group_external_payments_ready('00000000-0000-0000-0000-000000000000'::uuid),
    false,
    'Should report unready for an unknown group'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
