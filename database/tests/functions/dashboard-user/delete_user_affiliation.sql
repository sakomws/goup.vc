-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set affiliationID 'affd0000-0000-0000-0000-000000000001'
\set allianceID 'affd0000-0000-0000-0000-000000000002'
\set entryID 'affd0000-0000-0000-0000-000000000003'
\set otherUserAffiliationID 'affd0000-0000-0000-0000-000000000004'
\set otherUserID 'affd0000-0000-0000-0000-000000000005'
\set userID 'affd0000-0000-0000-0000-000000000006'

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

-- Users
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
), (
    :'otherUserID',
    gen_random_bytes(32),
    'bob@example.com',
    true,
    'bob',
    'Bob'
);

-- Landscape entry
insert into landscape_entry (
    landscape_entry_id,
    alliance_id,
    added_by_user_id,
    name,
    slug,
    summary,
    kind
) values (
    :'entryID',
    :'allianceID',
    :'userID',
    'Acme Robotics',
    'acme-robotics',
    'Robotics startup',
    'startup'
);

-- Affiliations
insert into user_affiliation (user_affiliation_id, user_id, landscape_entry_id, role)
values
    (:'affiliationID', :'userID', :'entryID', 'founder'),
    (:'otherUserAffiliationID', :'otherUserID', :'entryID', 'maintainer');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Delete affiliation
select lives_ok(
    format(
        'select delete_user_affiliation(%L::uuid, %L::uuid)',
        :'userID',
        :'affiliationID'
    ),
    'Should execute delete_user_affiliation successfully'
);

-- Should delete affiliation row
select is(
    (select count(*) from user_affiliation where user_affiliation_id = :'affiliationID'::uuid),
    0::bigint,
    'Should remove affiliation record'
);

-- Should create the expected audit row
select results_eq(
    $$
        select
            action,
            actor_user_id,
            actor_username,
            resource_type,
            resource_id
        from audit_log
    $$,
    format(
        $$
        values (
            'user_affiliation_deleted',
            %L::uuid,
            'alice',
            'user_affiliation',
            %L::uuid
        )
        $$,
        :'userID',
        :'affiliationID'
    ),
    'Should create the expected audit row'
);

-- Should not delete other users' affiliations
select throws_ok(
    format(
        'select delete_user_affiliation(%L::uuid, %L::uuid)',
        :'userID',
        :'otherUserAffiliationID'
    ),
    'affiliation not found',
    'Should not delete other users affiliations'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
