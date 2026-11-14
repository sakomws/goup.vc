-- Tests event-level external-payments readiness.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set communityID 'e0790000-0000-0000-0000-000000000001'
\set eventCategoryID 'e0790000-0000-0000-0000-000000000002'
\set eventID 'e0790000-0000-0000-0000-000000000003'
\set eventNoUrlID 'e0790000-0000-0000-0000-000000000008'
\set eventUnreadyID 'e0790000-0000-0000-0000-000000000004'
\set groupCategoryID 'e0790000-0000-0000-0000-000000000005'
\set groupID 'e0790000-0000-0000-0000-000000000006'
\set groupUnreadyID 'e0790000-0000-0000-0000-000000000007'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Operator allowlist used by the ready-event scenario
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
select fx_event_category(:'eventCategoryID', :'communityID');
select fx_group_category(:'groupCategoryID', :'communityID');
select fx_group(
    :'groupID', :'communityID', :'groupCategoryID',
    jsonb_build_object('country_code', 'KR', 'external_payments_enabled', true)
);
select fx_group(
    :'groupUnreadyID', :'communityID', :'groupCategoryID',
    jsonb_build_object('country_code', 'US', 'external_payments_enabled', true)
);

-- External-marked event on the allowlisted group
insert into event (
    description,
    event_category_id,
    event_id,
    event_kind_id,
    external_payment_url,
    group_id,
    name,
    slug,
    timezone
) values (
    'Ready event',
    :'eventCategoryID',
    :'eventID',
    'in-person',
    'https://pay.example.test/ready',
    :'groupID',
    'Ready Event',
    'ready-event',
    'UTC'
);

-- Allowlisted event without an external payment URL
insert into event (
    description,
    event_category_id,
    event_id,
    event_kind_id,
    group_id,
    name,
    slug,
    timezone
) values (
    'No URL event',
    :'eventCategoryID',
    :'eventNoUrlID',
    'in-person',
    :'groupID',
    'No URL Event',
    'no-url-event',
    'UTC'
);

-- External-marked event on a group that is not allowlisted
insert into event (
    description,
    event_category_id,
    event_id,
    event_kind_id,
    external_payment_url,
    group_id,
    name,
    slug,
    timezone
) values (
    'Unready event',
    :'eventCategoryID',
    :'eventUnreadyID',
    'in-person',
    'https://pay.example.test/unready',
    :'groupUnreadyID',
    'Unready Event',
    'unready-event',
    'UTC'
);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should report ready when the toggle, URL, and allowlist all match
select is(
    is_event_external_payments_ready(:'eventID'::uuid),
    true,
    'Should report ready when the toggle, URL, and allowlist all match'
);

-- Should report unready when the event has no external payment URL
select is(
    is_event_external_payments_ready(:'eventNoUrlID'::uuid),
    false,
    'Should report unready when the event has no external payment URL'
);

-- Should report unready when the group country is not allowlisted
select is(
    is_event_external_payments_ready(:'eventUnreadyID'::uuid),
    false,
    'Should report unready when the group country is not allowlisted'
);

-- Should report unready for an unknown event
select is(
    is_event_external_payments_ready('00000000-0000-0000-0000-000000000000'::uuid),
    false,
    'Should report unready for an unknown event'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
