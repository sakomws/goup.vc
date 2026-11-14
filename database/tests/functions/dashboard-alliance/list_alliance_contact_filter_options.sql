-- Tests listing the alliance contact filter options.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(6);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set activeParentGroupID '8a0e0000-0000-0000-0000-000000000024'
\set activeSubgroupID '8a0e0000-0000-0000-0000-000000000025'
\set alphaCategoryID '8a0e0000-0000-0000-0000-000000000004'
\set allianceID '8a0e0000-0000-0000-0000-000000000001'
\set deletedGroupID '8a0e0000-0000-0000-0000-000000000016'
\set deletedParentGroupID '8a0e0000-0000-0000-0000-000000000026'
\set deletedParentSubgroupID '8a0e0000-0000-0000-0000-000000000027'
\set deletedSubgroupID '8a0e0000-0000-0000-0000-000000000028'
\set eastRegionID '8a0e0000-0000-0000-0000-000000000008'
\set emptyCategoryID '8a0e0000-0000-0000-0000-000000000005'
\set emptyAllianceID '8a0e0000-0000-0000-0000-000000000002'
\set foreignCategoryID '8a0e0000-0000-0000-0000-000000000007'
\set foreignAllianceID '8a0e0000-0000-0000-0000-000000000003'
\set foreignGroupID '8a0e0000-0000-0000-0000-000000000017'
\set foreignRegionID '8a0e0000-0000-0000-0000-000000000011'
\set foreignSubgroupID '8a0e0000-0000-0000-0000-000000000029'
\set inactiveGroupID '8a0e0000-0000-0000-0000-000000000015'
\set inactiveParentGroupID '8a0e0000-0000-0000-0000-000000000030'
\set inactiveParentSubgroupID '8a0e0000-0000-0000-0000-000000000031'
\set inactiveSubgroupID '8a0e0000-0000-0000-0000-000000000032'
\set noRegionGroupID '8a0e0000-0000-0000-0000-000000000013'
\set regionGroupID '8a0e0000-0000-0000-0000-000000000012'
\set subgroupsCategoryID '8a0e0000-0000-0000-0000-000000000023'
\set subgroupsAllianceID '8a0e0000-0000-0000-0000-000000000020'
\set unavailableParentsCategoryID '8a0e0000-0000-0000-0000-000000000021'
\set unavailableParentsAllianceID '8a0e0000-0000-0000-0000-000000000018'
\set unavailableSubgroupsCategoryID '8a0e0000-0000-0000-0000-000000000022'
\set unavailableSubgroupsAllianceID '8a0e0000-0000-0000-0000-000000000019'
\set unavailableSubgroupsParentGroupID '8a0e0000-0000-0000-0000-000000000033'
\set vacantRegionID '8a0e0000-0000-0000-0000-000000000009'
\set zetaCategoryID '8a0e0000-0000-0000-0000-000000000006'
\set zetaGroupID '8a0e0000-0000-0000-0000-000000000014'
\set zuluRegionID '8a0e0000-0000-0000-0000-000000000010'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance whose filter options are listed
select fx_alliance(:'allianceID');

-- Alliance without group categories or regions
select fx_alliance(:'emptyAllianceID');

-- Alliance whose options are never listed
select fx_alliance(:'foreignAllianceID');

-- Alliance with an active subgroup of an active parent group
select fx_alliance(:'subgroupsAllianceID');

-- Alliance whose active subgroups have inactive or deleted parent groups
select fx_alliance(:'unavailableParentsAllianceID');

-- Alliance whose subgroups are inactive or deleted
select fx_alliance(:'unavailableSubgroupsAllianceID');

-- Unordered group category with active and unavailable groups
select fx_group_category(:'alphaCategoryID', :'allianceID', jsonb_build_object('name', 'Alpha'));

-- Unordered group category without groups
select fx_group_category(:'emptyCategoryID', :'allianceID', jsonb_build_object('name', 'Empty'));

-- Group category of the foreign alliance
select fx_group_category(:'foreignCategoryID', :'foreignAllianceID', jsonb_build_object('name', 'Foreign'));

-- Group category of the alliance with an active subgroup
select fx_group_category(:'subgroupsCategoryID', :'subgroupsAllianceID');

-- Group category of the alliance with inactive or deleted parent groups
select fx_group_category(:'unavailableParentsCategoryID', :'unavailableParentsAllianceID');

-- Group category of the alliance with inactive or deleted subgroups
select fx_group_category(:'unavailableSubgroupsCategoryID', :'unavailableSubgroupsAllianceID');

-- Ordered group category listed first
select fx_group_category(
    :'zetaCategoryID',
    :'allianceID',
    jsonb_build_object('name', 'Zeta', 'order', 1)
);

-- Unordered region with active and inactive groups
insert into region (region_id, alliance_id, name)
values (:'eastRegionID', :'allianceID', 'East');

-- Region of the foreign alliance
insert into region (region_id, alliance_id, name)
values (:'foreignRegionID', :'foreignAllianceID', 'Foreign');

-- Unordered region without groups
insert into region (region_id, alliance_id, name)
values (:'vacantRegionID', :'allianceID', 'Vacant');

-- Ordered region listed first
insert into region (region_id, alliance_id, name, "order")
values (:'zuluRegionID', :'allianceID', 'Zulu', 1);

-- Active parent group of the alliance with an active subgroup
select fx_group(:'activeParentGroupID', :'subgroupsAllianceID', :'subgroupsCategoryID');

-- Deleted group without region, excluded from counts and group options
select fx_group(
    :'deletedGroupID',
    :'allianceID',
    :'alphaCategoryID',
    jsonb_build_object('active', false, 'deleted', true, 'name', 'Deleted')
);

-- Parent group deleted once its subgroup is linked
select fx_group(:'deletedParentGroupID', :'unavailableParentsAllianceID', :'unavailableParentsCategoryID');

-- Group of the foreign alliance without region, parent of its active subgroup
select fx_group(:'foreignGroupID', :'foreignAllianceID', :'foreignCategoryID');

-- Inactive group in the east region, excluded from counts and group options
select fx_group(
    :'inactiveGroupID',
    :'allianceID',
    :'alphaCategoryID',
    jsonb_build_object('active', false, 'name', 'Inactive', 'region_id', :'eastRegionID')
);

-- Parent group deactivated once its subgroup is linked
select fx_group(:'inactiveParentGroupID', :'unavailableParentsAllianceID', :'unavailableParentsCategoryID');

-- Active group without region
select fx_group(:'noRegionGroupID', :'allianceID', :'alphaCategoryID', jsonb_build_object('name', 'Rust'));

-- Active group in the east region
select fx_group(
    :'regionGroupID',
    :'allianceID',
    :'alphaCategoryID',
    jsonb_build_object('name', 'Kubernetes', 'region_id', :'eastRegionID')
);

-- Active parent group of the inactive and deleted subgroups
select fx_group(
    :'unavailableSubgroupsParentGroupID',
    :'unavailableSubgroupsAllianceID',
    :'unavailableSubgroupsCategoryID'
);

-- Active group in the zulu region
select fx_group(
    :'zetaGroupID',
    :'allianceID',
    :'zetaCategoryID',
    jsonb_build_object('name', 'Go', 'region_id', :'zuluRegionID')
);

-- Active subgroup of the active parent group
select fx_group(
    :'activeSubgroupID',
    :'subgroupsAllianceID',
    :'subgroupsCategoryID',
    jsonb_build_object('parent_group_id', :'activeParentGroupID')
);

-- Active subgroup of the parent group deleted below
select fx_group(
    :'deletedParentSubgroupID',
    :'unavailableParentsAllianceID',
    :'unavailableParentsCategoryID',
    jsonb_build_object('parent_group_id', :'deletedParentGroupID')
);

-- Deleted subgroup of an active parent group
select fx_group(
    :'deletedSubgroupID',
    :'unavailableSubgroupsAllianceID',
    :'unavailableSubgroupsCategoryID',
    jsonb_build_object(
        'active', false,
        'deleted', true,
        'parent_group_id', :'unavailableSubgroupsParentGroupID'
    )
);

-- Active subgroup of the foreign alliance, never flagged for other alliances
select fx_group(
    :'foreignSubgroupID',
    :'foreignAllianceID',
    :'foreignCategoryID',
    jsonb_build_object('parent_group_id', :'foreignGroupID')
);

-- Active subgroup of the parent group deactivated below
select fx_group(
    :'inactiveParentSubgroupID',
    :'unavailableParentsAllianceID',
    :'unavailableParentsCategoryID',
    jsonb_build_object('parent_group_id', :'inactiveParentGroupID')
);

-- Inactive subgroup of an active parent group
select fx_group(
    :'inactiveSubgroupID',
    :'unavailableSubgroupsAllianceID',
    :'unavailableSubgroupsCategoryID',
    jsonb_build_object('active', false, 'parent_group_id', :'unavailableSubgroupsParentGroupID')
);

-- Delete the linked parent group, since the parent check rejects links to deleted parents
update "group"
set
    active = false,
    deleted = true
where group_id = :'deletedParentGroupID';

-- Deactivate the linked parent group, since the parent check rejects links to inactive parents
update "group"
set active = false
where group_id = :'inactiveParentGroupID';

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should flag subgroups when an active group has an active parent group
select is(
    (list_alliance_contact_filter_options(:'subgroupsAllianceID'::uuid)::jsonb->>'has_subgroups')::boolean,
    true,
    'Should flag subgroups when an active group has an active parent group'
);

-- Should ignore inactive and deleted subgroups
select is(
    (list_alliance_contact_filter_options(:'unavailableSubgroupsAllianceID'::uuid)::jsonb->>'has_subgroups')::boolean,
    false,
    'Should ignore inactive and deleted subgroups'
);

-- Should ignore subgroups of inactive and deleted parent groups
select is(
    (list_alliance_contact_filter_options(:'unavailableParentsAllianceID'::uuid)::jsonb->>'has_subgroups')::boolean,
    false,
    'Should ignore subgroups of inactive and deleted parent groups'
);

-- Should ignore subgroups of other alliances
select is(
    (list_alliance_contact_filter_options(:'allianceID'::uuid)::jsonb->>'has_subgroups')::boolean,
    false,
    'Should ignore subgroups of other alliances'
);

-- Should return empty options for an alliance without groups, categories or regions
select is(
    list_alliance_contact_filter_options(:'emptyAllianceID'::uuid)::jsonb,
    '{"group_categories": [], "groups": [], "has_subgroups": false, "no_region_groups_count": 0, "regions": []}'::jsonb,
    'Should return empty options for an alliance without groups, categories or regions'
);

-- Should return options with active groups labeled and counted in display order
select is(
    list_alliance_contact_filter_options(:'allianceID'::uuid)::jsonb,
    jsonb_build_object(
        'group_categories', jsonb_build_array(
            jsonb_build_object('group_category_id', :'zetaCategoryID', 'groups_count', 1, 'name', 'Zeta'),
            jsonb_build_object('group_category_id', :'alphaCategoryID', 'groups_count', 2, 'name', 'Alpha'),
            jsonb_build_object('group_category_id', :'emptyCategoryID', 'groups_count', 0, 'name', 'Empty')
        ),
        'groups', jsonb_build_array(
            jsonb_build_object('group_id', :'zetaGroupID', 'label', 'Go · Zeta · Zulu'),
            jsonb_build_object('group_id', :'regionGroupID', 'label', 'Kubernetes · Alpha · East'),
            jsonb_build_object('group_id', :'noRegionGroupID', 'label', 'Rust · Alpha')
        ),
        'has_subgroups', false,
        'no_region_groups_count', 1,
        'regions', jsonb_build_array(
            jsonb_build_object('groups_count', 1, 'name', 'Zulu', 'region_id', :'zuluRegionID'),
            jsonb_build_object('groups_count', 1, 'name', 'East', 'region_id', :'eastRegionID'),
            jsonb_build_object('groups_count', 0, 'name', 'Vacant', 'region_id', :'vacantRegionID')
        )
    ),
    'Should return options with active groups labeled and counted in display order'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
