-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID 'affc0000-0000-0000-0000-000000000001'
\set repoEntryID 'affc0000-0000-0000-0000-000000000002'
\set startupEntryID 'affc0000-0000-0000-0000-000000000003'
\set unpublishedEntryID 'affc0000-0000-0000-0000-000000000004'
\set userID 'affc0000-0000-0000-0000-000000000005'

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
    'Alliance for testing landscape entry options',
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
    :'startupEntryID',
    :'allianceID',
    :'userID',
    'Acme Robotics',
    'acme-robotics',
    'Robotics startup',
    'startup',
    true
), (
    :'repoEntryID',
    :'allianceID',
    :'userID',
    'ocg-server',
    'ocg-server',
    'Community platform server',
    'github_project',
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

-- Should list published entries ordered by kind and name
select is(
    list_landscape_entry_options()::jsonb,
    format(
        $$
        [
            {
                "landscape_entry_id": "%s",
                "name": "ocg-server",
                "kind": "github_project"
            },
            {
                "landscape_entry_id": "%s",
                "name": "Acme Robotics",
                "kind": "startup"
            }
        ]
        $$,
        :'repoEntryID',
        :'startupEntryID'
    )::jsonb,
    'Should list published entries ordered by kind and name'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
