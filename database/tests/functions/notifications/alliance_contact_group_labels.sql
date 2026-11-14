-- Tests labeling alliance groups for the contact filters.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(7);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set alphaGroupID '8a130000-0000-0000-0000-000000000011'
\set alphaNoRegionGroupID '8a130000-0000-0000-0000-000000000012'
\set bravoGroupID '8a130000-0000-0000-0000-000000000013'
\set bravoLookAlikeGroupID '8a130000-0000-0000-0000-000000000014'
\set charlieDeletedLookAlikeGroupID '8a130000-0000-0000-0000-000000000015'
\set charlieGroupID '8a130000-0000-0000-0000-000000000016'
\set allianceID '8a130000-0000-0000-0000-000000000001'
\set deltaGroupID '8a130000-0000-0000-0000-000000000018'
\set deltaLookAlikeGroupID '8a130000-0000-0000-0000-000000000019'
\set echoDeletedGroupID '8a130000-0000-0000-0000-000000000020'
\set foreignCategoryID '8a130000-0000-0000-0000-000000000004'
\set foreignAllianceID '8a130000-0000-0000-0000-000000000002'
\set foreignGroupID '8a130000-0000-0000-0000-000000000017'
\set foxtrotGroupID '8a130000-0000-0000-0000-000000000021'
\set foxtrotPlatformGroupID '8a130000-0000-0000-0000-000000000022'
\set northRegionID '8a130000-0000-0000-0000-000000000005'
\set platformCategoryID '8a130000-0000-0000-0000-000000000006'
\set southCategoryID '8a130000-0000-0000-0000-000000000007'
\set southRegionID '8a130000-0000-0000-0000-000000000008'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance whose groups are labeled
select fx_alliance(:'allianceID');

-- Alliance whose groups are never labeled
select fx_alliance(:'foreignAllianceID');

-- Group category of the foreign alliance, named like the platform category
select fx_group_category(:'foreignCategoryID', :'foreignAllianceID', jsonb_build_object('name', 'Platform'));

-- Platform group category
select fx_group_category(:'platformCategoryID', :'allianceID', jsonb_build_object('name', 'Platform'));

-- Group category named like the south region
select fx_group_category(:'southCategoryID', :'allianceID', jsonb_build_object('name', 'South'));

-- North region
insert into region (region_id, alliance_id, name)
values (:'northRegionID', :'allianceID', 'North');

-- South region
insert into region (region_id, alliance_id, name)
values (:'southRegionID', :'allianceID', 'South');

-- Platform group in the north region
select fx_group(
    :'alphaGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object('name', 'Alpha', 'region_id', :'northRegionID', 'slug', 'alpha')
);

-- Platform group sharing the alpha name without region
select fx_group(
    :'alphaNoRegionGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object('name', 'Alpha', 'slug', 'alpha-no-region')
);

-- Platform group in the north region with a pretty slug
select fx_group(
    :'bravoGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object(
        'name', 'Bravo',
        'region_id', :'northRegionID',
        'slug', 'bravo-x1',
        'slug_pretty', 'bravo-one'
    )
);

-- Inactive look-alike of the bravo group
select fx_group(
    :'bravoLookAlikeGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object('active', false, 'name', 'Bravo', 'region_id', :'northRegionID', 'slug', 'bravo-two')
);

-- Deleted look-alike of the charlie group
select fx_group(
    :'charlieDeletedLookAlikeGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object(
        'active', false,
        'deleted', true,
        'name', 'Charlie',
        'region_id', :'northRegionID',
        'slug', 'charlie-old'
    )
);

-- Platform group in the north region
select fx_group(
    :'charlieGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object('name', 'Charlie', 'region_id', :'northRegionID', 'slug', 'charlie')
);

-- Platform group without region
select fx_group(
    :'deltaGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object('name', 'Delta', 'slug', 'delta')
);

-- Look-alike of the delta group without region
select fx_group(
    :'deltaLookAlikeGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object('name', 'Delta', 'slug', 'delta-two')
);

-- Deleted platform group in the north region without peers
select fx_group(
    :'echoDeletedGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object(
        'active', false,
        'deleted', true,
        'name', 'Echo',
        'region_id', :'northRegionID',
        'slug', 'echo-old'
    )
);

-- Platform group in the south region whose base label reads "Foxtrot · Platform · South"
select fx_group(
    :'foxtrotGroupID',
    :'allianceID',
    :'platformCategoryID',
    jsonb_build_object('name', 'Foxtrot', 'region_id', :'southRegionID', 'slug', 'foxtrot')
);

-- South category group without region whose separator-containing name builds the same base label
select fx_group(
    :'foxtrotPlatformGroupID',
    :'allianceID',
    :'southCategoryID',
    jsonb_build_object('name', 'Foxtrot · Platform', 'slug', 'foxtrot-platform')
);

-- Group of the foreign alliance sharing the base label of the alpha group without region
select fx_group(
    :'foreignGroupID',
    :'foreignAllianceID',
    :'foreignCategoryID',
    jsonb_build_object('name', 'Alpha', 'slug', 'alpha')
);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should append the pretty slug instead of the slug when set
select results_eq(
    format(
        $$
            select label
            from alliance_contact_group_labels(%L::uuid)
            where group_id = %L::uuid
        $$,
        :'allianceID',
        :'bravoGroupID'
    ),
    $$ values ('Bravo · Platform · North (bravo-one)') $$,
    'Should append the pretty slug instead of the slug when set'
);

-- Should append the slug to deleted groups sharing their base label
select results_eq(
    format(
        $$
            select label
            from alliance_contact_group_labels(%L::uuid)
            where group_id = %L::uuid
        $$,
        :'allianceID',
        :'charlieDeletedLookAlikeGroupID'
    ),
    $$ values ('Charlie · Platform · North (charlie-old)') $$,
    'Should append the slug to deleted groups sharing their base label'
);

-- Should append the slug to groups sharing their base label
select results_eq(
    format(
        $$
            select group_id, label
            from alliance_contact_group_labels(%L::uuid)
            where group_id = any(array[%L, %L, %L, %L, %L]::uuid[])
            order by group_id
        $$,
        :'allianceID',
        :'bravoLookAlikeGroupID',
        :'deltaGroupID',
        :'deltaLookAlikeGroupID',
        :'foxtrotGroupID',
        :'foxtrotPlatformGroupID'
    ),
    format(
        $$ values
            (%L::uuid, 'Bravo · Platform · North (bravo-two)'),
            (%L::uuid, 'Delta · Platform (delta)'),
            (%L::uuid, 'Delta · Platform (delta-two)'),
            (%L::uuid, 'Foxtrot · Platform · South (foxtrot)'),
            (%L::uuid, 'Foxtrot · Platform · South (foxtrot-platform)')
        $$,
        :'bravoLookAlikeGroupID',
        :'deltaGroupID',
        :'deltaLookAlikeGroupID',
        :'foxtrotGroupID',
        :'foxtrotPlatformGroupID'
    ),
    'Should append the slug to groups sharing their base label'
);

-- Should label groups with their name, category and region, omitting a missing region
select results_eq(
    format(
        $$
            select group_id, label
            from alliance_contact_group_labels(%L::uuid)
            where group_id = any(array[%L, %L]::uuid[])
            order by group_id
        $$,
        :'allianceID',
        :'alphaGroupID',
        :'alphaNoRegionGroupID'
    ),
    format(
        $$ values
            (%L::uuid, 'Alpha · Platform · North'),
            (%L::uuid, 'Alpha · Platform')
        $$,
        :'alphaGroupID',
        :'alphaNoRegionGroupID'
    ),
    'Should label groups with their name, category and region, omitting a missing region'
);

-- Should not append the slug to deleted groups without peers
select results_eq(
    format(
        $$
            select label
            from alliance_contact_group_labels(%L::uuid)
            where group_id = %L::uuid
        $$,
        :'allianceID',
        :'echoDeletedGroupID'
    ),
    $$ values ('Echo · Platform · North') $$,
    'Should not append the slug to deleted groups without peers'
);

-- Should not let deleted look-alikes append the slug to current groups
select results_eq(
    format(
        $$
            select label
            from alliance_contact_group_labels(%L::uuid)
            where group_id = %L::uuid
        $$,
        :'allianceID',
        :'charlieGroupID'
    ),
    $$ values ('Charlie · Platform · North') $$,
    'Should not let deleted look-alikes append the slug to current groups'
);

-- Should return every group of the alliance only, including deleted ones
select results_eq(
    format(
        $$
            select group_id
            from alliance_contact_group_labels(%L::uuid)
            order by group_id
        $$,
        :'allianceID'
    ),
    format(
        $$ values
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid),
            (%L::uuid)
        $$,
        :'alphaGroupID',
        :'alphaNoRegionGroupID',
        :'bravoGroupID',
        :'bravoLookAlikeGroupID',
        :'charlieDeletedLookAlikeGroupID',
        :'charlieGroupID',
        :'deltaGroupID',
        :'deltaLookAlikeGroupID',
        :'echoDeletedGroupID',
        :'foxtrotGroupID',
        :'foxtrotPlatformGroupID'
    ),
    'Should return every group of the alliance only, including deleted ones'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
