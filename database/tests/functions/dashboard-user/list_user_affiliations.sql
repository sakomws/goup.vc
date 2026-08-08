-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set affiliationRepoID 'affe0000-0000-0000-0000-000000000001'
\set affiliationStartupID 'affe0000-0000-0000-0000-000000000002'
\set allianceID 'affe0000-0000-0000-0000-000000000003'
\set otherUserID 'affe0000-0000-0000-0000-000000000004'
\set repoEntryID 'affe0000-0000-0000-0000-000000000005'
\set startupEntryID 'affe0000-0000-0000-0000-000000000006'
\set userID 'affe0000-0000-0000-0000-000000000007'

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

-- Landscape entries
insert into landscape_entry (
    landscape_entry_id,
    alliance_id,
    added_by_user_id,
    name,
    slug,
    summary,
    kind,
    logo_url,
    website_url,
    github_url
) values (
    :'startupEntryID',
    :'allianceID',
    :'userID',
    'Acme Robotics',
    'acme-robotics',
    'Robotics startup',
    'startup',
    'https://example.com/acme.png',
    'https://acme.example.com',
    null
), (
    :'repoEntryID',
    :'allianceID',
    :'userID',
    'ocg-server',
    'ocg-server',
    'Community platform server',
    'github_project',
    null,
    null,
    'https://github.com/example/ocg-server'
);

-- Affiliations
insert into user_affiliation (user_affiliation_id, user_id, landscape_entry_id, role)
values
    (:'affiliationStartupID', :'userID', :'startupEntryID', 'founder'),
    (:'affiliationRepoID', :'userID', :'repoEntryID', 'maintainer');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should list the user's affiliations ordered by kind and name
select is(
    list_user_affiliations(:'userID'::uuid)::jsonb,
    format(
        $$
        [
            {
                "user_affiliation_id": "%s",
                "landscape_entry_id": "%s",
                "entry_name": "ocg-server",
                "entry_kind": "github_project",
                "entry_logo_url": null,
                "entry_website_url": null,
                "entry_github_url": "https://github.com/example/ocg-server",
                "role": "maintainer"
            },
            {
                "user_affiliation_id": "%s",
                "landscape_entry_id": "%s",
                "entry_name": "Acme Robotics",
                "entry_kind": "startup",
                "entry_logo_url": "https://example.com/acme.png",
                "entry_website_url": "https://acme.example.com",
                "entry_github_url": null,
                "role": "founder"
            }
        ]
        $$,
        :'affiliationRepoID',
        :'repoEntryID',
        :'affiliationStartupID',
        :'startupEntryID'
    )::jsonb,
    'Should list the user affiliations ordered by kind and name'
);

-- Should return an empty array for users without affiliations
select is(
    list_user_affiliations(:'otherUserID'::uuid)::jsonb,
    '[]'::jsonb,
    'Should return an empty array for users without affiliations'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
