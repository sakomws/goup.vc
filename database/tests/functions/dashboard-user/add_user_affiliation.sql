-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(7);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID 'affa0000-0000-0000-0000-000000000001'
\set entryID 'affa0000-0000-0000-0000-000000000002'
\set unpublishedEntryID 'affa0000-0000-0000-0000-000000000003'
\set userID 'affa0000-0000-0000-0000-000000000004'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance
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
    'affiliation-alliance',
    'Affiliation Alliance',
    'Alliance for testing user affiliations',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

-- User
insert into "user" (
    user_id,
    auth_hash,
    email,
    email_verified,
    username,
    name
) values (
    :'userID',
    gen_random_bytes(32),
    'alice@example.com',
    true,
    'alice',
    'Alice'
);

-- Landscape entries
insert into landscape_entry (
    landscape_entry_id,
    alliance_id,
    added_by_user_id,
    name,
    slug,
    summary,
    kind,
    published
) values (
    :'entryID',
    :'allianceID',
    :'userID',
    'Acme Robotics',
    'acme-robotics',
    'Robotics startup',
    'startup',
    true
), (
    :'unpublishedEntryID',
    :'allianceID',
    :'userID',
    'Hidden Startup',
    'hidden-startup',
    'Unpublished startup',
    'startup',
    false
);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Add affiliation
select lives_ok(
    format(
        'select add_user_affiliation(%L::uuid, %L::uuid, %L)',
        :'userID',
        :'entryID',
        'founder'
    ),
    'Should execute add_user_affiliation successfully'
);

-- Should create affiliation row
select is(
    (select role from user_affiliation where user_id = :'userID'::uuid and landscape_entry_id = :'entryID'::uuid),
    'founder',
    'Should create affiliation with the requested role'
);

-- Adding again should update the role instead of failing
select lives_ok(
    format(
        'select add_user_affiliation(%L::uuid, %L::uuid, %L)',
        :'userID',
        :'entryID',
        'maintainer'
    ),
    'Should update the role when the affiliation already exists'
);

-- Should keep a single row with the updated role
select results_eq(
    format(
        $$
        select role, count(*)
        from user_affiliation
        where user_id = %L::uuid and landscape_entry_id = %L::uuid
        group by role
        $$,
        :'userID',
        :'entryID'
    ),
    $$ values ('maintainer', 1::bigint) $$,
    'Should keep a single affiliation row with the updated role'
);

-- Should create the expected audit rows
select results_eq(
    $$
        select
            action,
            actor_user_id,
            actor_username,
            resource_type
        from audit_log
        order by created_at
    $$,
    format(
        $$
        values
            ('user_affiliation_added', %L::uuid, 'alice', 'user_affiliation'),
            ('user_affiliation_added', %L::uuid, 'alice', 'user_affiliation')
        $$,
        :'userID',
        :'userID'
    ),
    'Should create the expected audit rows'
);

-- Should reject invalid roles
select throws_ok(
    format(
        'select add_user_affiliation(%L::uuid, %L::uuid, %L)',
        :'userID',
        :'entryID',
        'ceo'
    ),
    'invalid affiliation role: ceo',
    'Should reject invalid roles'
);

-- Should reject unpublished entries
select throws_ok(
    format(
        'select add_user_affiliation(%L::uuid, %L::uuid, %L)',
        :'userID',
        :'unpublishedEntryID',
        'founder'
    ),
    'landscape entry not found',
    'Should reject unpublished entries'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
