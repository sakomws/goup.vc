-- Tests listing the alliance groups the contact feature can reach.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set categoryID '8a140000-0000-0000-0000-000000000003'
\set allianceID '8a140000-0000-0000-0000-000000000001'
\set deletedGroupID '8a140000-0000-0000-0000-000000000013'
\set foreignCategoryID '8a140000-0000-0000-0000-000000000004'
\set foreignAllianceID '8a140000-0000-0000-0000-000000000002'
\set foreignGroupID '8a140000-0000-0000-0000-000000000014'
\set groupID '8a140000-0000-0000-0000-000000000011'
\set inactiveGroupID '8a140000-0000-0000-0000-000000000012'
\set regionID '8a140000-0000-0000-0000-000000000005'
\set subgroupID '8a140000-0000-0000-0000-000000000015'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance whose groups are listed
select fx_alliance(:'allianceID');

-- Alliance whose groups are never listed
select fx_alliance(:'foreignAllianceID');

-- Group category of the alliance
select fx_group_category(:'categoryID', :'allianceID');

-- Group category of the foreign alliance
select fx_group_category(:'foreignCategoryID', :'foreignAllianceID');

-- Region of the alliance
insert into region (region_id, alliance_id, name)
values (:'regionID', :'allianceID', 'North');

-- Deleted group of the alliance
select fx_group(
    :'deletedGroupID',
    :'allianceID',
    :'categoryID',
    jsonb_build_object('active', false, 'deleted', true)
);

-- Active group of the foreign alliance
select fx_group(:'foreignGroupID', :'foreignAllianceID', :'foreignCategoryID');

-- Active group of the alliance in the region
select fx_group(:'groupID', :'allianceID', :'categoryID', jsonb_build_object('region_id', :'regionID'));

-- Inactive group of the alliance
select fx_group(:'inactiveGroupID', :'allianceID', :'categoryID', jsonb_build_object('active', false));

-- Active subgroup of the active group without region
select fx_group(:'subgroupID', :'allianceID', :'categoryID', jsonb_build_object('parent_group_id', :'groupID'));

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return the active non-deleted groups of the alliance with their category, parent and region
select results_eq(
    format(
        $$
            select group_category_id, group_id, parent_group_id, region_id
            from alliance_contact_groups(%L::uuid)
            order by group_id
        $$,
        :'allianceID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid, null::uuid, %L::uuid),
            (%L::uuid, %L::uuid, %L::uuid, null::uuid)
        $$,
        :'categoryID', :'groupID', :'regionID',
        :'categoryID', :'subgroupID', :'groupID'
    ),
    'Should return the active non-deleted groups of the alliance with their category, parent and region'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
