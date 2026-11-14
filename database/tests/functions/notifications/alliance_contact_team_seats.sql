-- Tests matching alliance group team seats against contact filters.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(13);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set adminUserID '8a0c0000-0000-0000-0000-000000000021'
\set categoryAID '8a0c0000-0000-0000-0000-000000000002'
\set categoryBID '8a0c0000-0000-0000-0000-000000000003'
\set allianceID '8a0c0000-0000-0000-0000-000000000001'
\set deletedGroupID '8a0c0000-0000-0000-0000-000000000016'
\set deletedGroupUserID '8a0c0000-0000-0000-0000-000000000029'
\set foreignCategoryID '8a0c0000-0000-0000-0000-000000000005'
\set foreignAllianceID '8a0c0000-0000-0000-0000-000000000004'
\set foreignGroupID '8a0c0000-0000-0000-0000-000000000017'
\set foreignUserID '8a0c0000-0000-0000-0000-000000000030'
\set groupAID '8a0c0000-0000-0000-0000-000000000011'
\set groupBID '8a0c0000-0000-0000-0000-000000000012'
\set inactiveGroupID '8a0c0000-0000-0000-0000-000000000015'
\set inactiveGroupUserID '8a0c0000-0000-0000-0000-000000000028'
\set mixedUserID '8a0c0000-0000-0000-0000-000000000022'
\set noRegionGroupID '8a0c0000-0000-0000-0000-000000000013'
\set noRegionUserID '8a0c0000-0000-0000-0000-000000000024'
\set northRegionID '8a0c0000-0000-0000-0000-000000000006'
\set pendingUserID '8a0c0000-0000-0000-0000-000000000026'
\set southRegionID '8a0c0000-0000-0000-0000-000000000007'
\set subgroupID '8a0c0000-0000-0000-0000-000000000014'
\set subgroupUserID '8a0c0000-0000-0000-0000-000000000025'
\set unverifiedUserID '8a0c0000-0000-0000-0000-000000000027'
\set viewerUserID '8a0c0000-0000-0000-0000-000000000023'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance whose group team seats are matched
select fx_alliance(:'allianceID');

-- Alliance whose seats are never matched
select fx_alliance(:'foreignAllianceID');

-- Admin of group A
select fx_user(:'adminUserID');

-- Admin of the deleted group
select fx_user(:'deletedGroupUserID');

-- Admin of the foreign alliance group
select fx_user(:'foreignUserID');

-- Admin of the inactive group
select fx_user(:'inactiveGroupUserID');

-- Viewer of group A and admin of group B
select fx_user(:'mixedUserID');

-- Events manager of the group without region
select fx_user(:'noRegionUserID');

-- Admin of group A who has not accepted the invitation
select fx_user(:'pendingUserID');

-- Check-in manager of the subgroup
select fx_user(:'subgroupUserID');

-- Admin of group A without a verified email
select fx_user(:'unverifiedUserID', jsonb_build_object('email_verified', false));

-- Viewer of group B
select fx_user(:'viewerUserID');

-- Group category A of the alliance
select fx_group_category(:'categoryAID', :'allianceID');

-- Group category B of the alliance
select fx_group_category(:'categoryBID', :'allianceID');

-- Group category of the foreign alliance
select fx_group_category(:'foreignCategoryID', :'foreignAllianceID');

-- North region of the alliance
insert into region (region_id, alliance_id, name)
values (:'northRegionID', :'allianceID', 'North');

-- South region of the alliance
insert into region (region_id, alliance_id, name)
values (:'southRegionID', :'allianceID', 'South');

-- Deleted group in category A and the north region
select fx_group(
    :'deletedGroupID',
    :'allianceID',
    :'categoryAID',
    jsonb_build_object('active', false, 'deleted', true, 'region_id', :'northRegionID')
);

-- Group of the foreign alliance
select fx_group(:'foreignGroupID', :'foreignAllianceID', :'foreignCategoryID');

-- Active group A in category A and the north region
select fx_group(:'groupAID', :'allianceID', :'categoryAID', jsonb_build_object('region_id', :'northRegionID'));

-- Active group B in category B and the south region
select fx_group(:'groupBID', :'allianceID', :'categoryBID', jsonb_build_object('region_id', :'southRegionID'));

-- Inactive group in category A and the north region
select fx_group(
    :'inactiveGroupID',
    :'allianceID',
    :'categoryAID',
    jsonb_build_object('active', false, 'region_id', :'northRegionID')
);

-- Active group in category A without region
select fx_group(:'noRegionGroupID', :'allianceID', :'categoryAID');

-- Active subgroup of group A in category B and the north region
select fx_group(
    :'subgroupID',
    :'allianceID',
    :'categoryBID',
    jsonb_build_object('parent_group_id', :'groupAID', 'region_id', :'northRegionID')
);

-- Team seats covering every eligibility and filter scenario
insert into group_team (group_id, user_id, accepted, role) values
    (:'deletedGroupID', :'deletedGroupUserID', true, 'admin'),
    (:'foreignGroupID', :'foreignUserID', true, 'admin'),
    (:'groupAID', :'adminUserID', true, 'admin'),
    (:'groupAID', :'mixedUserID', true, 'viewer'),
    (:'groupAID', :'pendingUserID', false, 'admin'),
    (:'groupAID', :'unverifiedUserID', true, 'admin'),
    (:'groupBID', :'mixedUserID', true, 'admin'),
    (:'groupBID', :'viewerUserID', true, 'viewer'),
    (:'inactiveGroupID', :'inactiveGroupUserID', true, 'admin'),
    (:'noRegionGroupID', :'noRegionUserID', true, 'events-manager'),
    (:'subgroupID', :'subgroupUserID', true, 'check-in-manager');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should apply all filters to the same seat
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('group_category_ids', jsonb_build_array(%L), 'roles', jsonb_build_array('admin'))
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'categoryAID'
    ),
    format(
        $$ values (%L::uuid, %L::uuid) $$,
        :'groupAID',
        :'adminUserID'
    ),
    'Should apply all filters to the same seat'
);

-- Should apply the group filter together with the other filters
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object(
                    'group_category_ids', jsonb_build_array(%L, %L),
                    'group_ids', jsonb_build_array(%L, %L),
                    'regions', jsonb_build_array(%L, %L),
                    'roles', jsonb_build_array('admin', 'check-in-manager')
                )
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'categoryAID',
        :'categoryBID',
        :'groupAID',
        :'subgroupID',
        :'northRegionID',
        :'southRegionID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupAID', :'adminUserID',
        :'subgroupID', :'subgroupUserID'
    ),
    'Should apply the group filter together with the other filters'
);

-- Should filter by group category
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('group_category_ids', jsonb_build_array(%L))
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'categoryBID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupBID', :'mixedUserID',
        :'groupBID', :'viewerUserID',
        :'subgroupID', :'subgroupUserID'
    ),
    'Should filter by group category'
);

-- Should filter by group role
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                '{"roles": ["check-in-manager", "viewer"]}'::jsonb
            )
            order by group_id, user_id
        $$,
        :'allianceID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupAID', :'mixedUserID',
        :'groupBID', :'viewerUserID',
        :'subgroupID', :'subgroupUserID'
    ),
    'Should filter by group role'
);

-- Should filter by group without including subgroup teams
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('group_ids', jsonb_build_array(%L))
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'groupAID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupAID', :'adminUserID',
        :'groupAID', :'mixedUserID'
    ),
    'Should filter by group without including subgroup teams'
);

-- Should filter by no region only
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(%L::uuid, '{"regions": ["none"]}'::jsonb)
            order by group_id, user_id
        $$,
        :'allianceID'
    ),
    format(
        $$ values (%L::uuid, %L::uuid) $$,
        :'noRegionGroupID',
        :'noRegionUserID'
    ),
    'Should filter by no region only'
);

-- Should filter by region
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('regions', jsonb_build_array(%L))
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'northRegionID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupAID', :'adminUserID',
        :'groupAID', :'mixedUserID',
        :'subgroupID', :'subgroupUserID'
    ),
    'Should filter by region'
);

-- Should filter by regions and no region
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('regions', jsonb_build_array(%L, 'none'))
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'southRegionID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupBID', :'mixedUserID',
        :'groupBID', :'viewerUserID',
        :'noRegionGroupID', :'noRegionUserID'
    ),
    'Should filter by regions and no region'
);

-- Should filter by subgroup without including the parent group team
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('group_ids', jsonb_build_array(%L))
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'subgroupID'
    ),
    format(
        $$ values (%L::uuid, %L::uuid) $$,
        :'subgroupID',
        :'subgroupUserID'
    ),
    'Should filter by subgroup without including the parent group team'
);

-- Should match no seats for unavailable selected groups
select is_empty(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('group_ids', jsonb_build_array(%L, %L))
            )
        $$,
        :'allianceID',
        :'deletedGroupID',
        :'inactiveGroupID'
    ),
    'Should match no seats for unavailable selected groups'
);

-- Should return each seat once for repeated selected groups
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                jsonb_build_object('group_ids', jsonb_build_array(%L, %L))
            )
            order by group_id, user_id
        $$,
        :'allianceID',
        :'groupAID',
        :'groupAID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupAID', :'adminUserID',
        :'groupAID', :'mixedUserID'
    ),
    'Should return each seat once for repeated selected groups'
);

-- Should return every accepted and verified seat of available groups for missing filters
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(%L::uuid, '{}'::jsonb)
            order by group_id, user_id
        $$,
        :'allianceID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupAID', :'adminUserID',
        :'groupAID', :'mixedUserID',
        :'groupBID', :'mixedUserID',
        :'groupBID', :'viewerUserID',
        :'noRegionGroupID', :'noRegionUserID',
        :'subgroupID', :'subgroupUserID'
    ),
    'Should return every accepted and verified seat of available groups for missing filters'
);

-- Should treat empty filter lists as all values
select results_eq(
    format(
        $$
            select group_id, user_id
            from alliance_contact_team_seats(
                %L::uuid,
                '{"group_category_ids": [], "group_ids": [], "regions": [], "roles": []}'::jsonb
            )
            order by group_id, user_id
        $$,
        :'allianceID'
    ),
    format(
        $$ values
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid),
            (%L::uuid, %L::uuid)
        $$,
        :'groupAID', :'adminUserID',
        :'groupAID', :'mixedUserID',
        :'groupBID', :'mixedUserID',
        :'groupBID', :'viewerUserID',
        :'noRegionGroupID', :'noRegionUserID',
        :'subgroupID', :'subgroupUserID'
    ),
    'Should treat empty filter lists as all values'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
