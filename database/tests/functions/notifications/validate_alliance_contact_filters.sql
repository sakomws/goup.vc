-- Tests validating alliance contact filters.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(13);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID '8a0b0000-0000-0000-0000-000000000001'
\set deletedGroupCategoryID '8a0b0000-0000-0000-0000-000000000002'
\set deletedGroupID '8a0b0000-0000-0000-0000-000000000008'
\set foreignAllianceID '8a0b0000-0000-0000-0000-000000000003'
\set foreignGroupCategoryID '8a0b0000-0000-0000-0000-000000000004'
\set foreignGroupID '8a0b0000-0000-0000-0000-000000000009'
\set foreignRegionID '8a0b0000-0000-0000-0000-000000000005'
\set groupCategoryID '8a0b0000-0000-0000-0000-000000000006'
\set groupID '8a0b0000-0000-0000-0000-000000000010'
\set inactiveGroupID '8a0b0000-0000-0000-0000-000000000011'
\set regionID '8a0b0000-0000-0000-0000-000000000007'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance whose contact filters are validated
select fx_alliance(:'allianceID');

-- Alliance owning the foreign filter values
select fx_alliance(:'foreignAllianceID');

-- Group category of the validated alliance
select fx_group_category(:'groupCategoryID', :'allianceID');

-- Group category of the foreign alliance
select fx_group_category(:'foreignGroupCategoryID', :'foreignAllianceID');

-- Region of the foreign alliance
insert into region (region_id, alliance_id, name)
values (:'foreignRegionID', :'foreignAllianceID', 'Foreign Region');

-- Region of the validated alliance
insert into region (region_id, alliance_id, name)
values (:'regionID', :'allianceID', 'Local Region');

-- Deleted group of the validated alliance
select fx_group(
    :'deletedGroupID',
    :'allianceID',
    :'groupCategoryID',
    jsonb_build_object('active', false, 'deleted', true)
);

-- Group of the foreign alliance
select fx_group(:'foreignGroupID', :'foreignAllianceID', :'foreignGroupCategoryID');

-- Active group of the validated alliance
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');

-- Inactive group of the validated alliance
select fx_group(:'inactiveGroupID', :'allianceID', :'groupCategoryID', jsonb_build_object('active', false));

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should accept empty filter lists
select lives_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            '{"group_category_ids": [], "group_ids": [], "regions": [], "roles": []}'::jsonb
        )$$,
        :'allianceID'
    ),
    'Should accept empty filter lists'
);

-- Should accept filter values of the alliance
select lives_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            jsonb_build_object(
                'group_category_ids', jsonb_build_array(%L),
                'group_ids', jsonb_build_array(%L),
                'regions', jsonb_build_array(%L, 'none'),
                'roles', jsonb_build_array('admin', 'check-in-manager', 'events-manager', 'viewer')
            )
        )$$,
        :'allianceID',
        :'groupCategoryID',
        :'groupID',
        :'regionID'
    ),
    'Should accept filter values of the alliance'
);

-- Should accept inactive and deleted groups of the alliance
select lives_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            jsonb_build_object('group_ids', jsonb_build_array(%L, %L))
        )$$,
        :'allianceID',
        :'deletedGroupID',
        :'inactiveGroupID'
    ),
    'Should accept inactive and deleted groups of the alliance'
);

-- Should accept missing filter lists
select lives_ok(
    format(
        $$select validate_alliance_contact_filters(%L::uuid, '{}'::jsonb)$$,
        :'allianceID'
    ),
    'Should accept missing filter lists'
);

-- Should reject deleted group categories
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            jsonb_build_object('group_category_ids', jsonb_build_array(%L))
        )$$,
        :'allianceID',
        :'deletedGroupCategoryID'
    ),
    'OCG01',
    'group category not found',
    'Should reject deleted group categories'
);

-- Should reject group categories from another alliance
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            jsonb_build_object('group_category_ids', jsonb_build_array(%L))
        )$$,
        :'allianceID',
        :'foreignGroupCategoryID'
    ),
    'OCG01',
    'group category not found',
    'Should reject group categories from another alliance'
);

-- Should reject groups from another alliance
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            jsonb_build_object('group_ids', jsonb_build_array(%L, %L))
        )$$,
        :'allianceID',
        :'groupID',
        :'foreignGroupID'
    ),
    'OCG01',
    'group not found',
    'Should reject groups from another alliance'
);

-- Should reject malformed group category identifiers
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            '{"group_category_ids": ["not-a-uuid"]}'::jsonb
        )$$,
        :'allianceID'
    ),
    'OCG01',
    'group category not found',
    'Should reject malformed group category identifiers'
);

-- Should reject malformed group identifiers
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            '{"group_ids": ["not-a-uuid"]}'::jsonb
        )$$,
        :'allianceID'
    ),
    'OCG01',
    'group not found',
    'Should reject malformed group identifiers'
);

-- Should reject malformed region values
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            '{"regions": ["nowhere"]}'::jsonb
        )$$,
        :'allianceID'
    ),
    'OCG01',
    'invalid region',
    'Should reject malformed region values'
);

-- Should reject mixed valid and foreign filter values
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            jsonb_build_object(
                'group_category_ids', jsonb_build_array(%L),
                'regions', jsonb_build_array(%L, %L)
            )
        )$$,
        :'allianceID',
        :'groupCategoryID',
        :'regionID',
        :'foreignRegionID'
    ),
    'OCG01',
    'region not found',
    'Should reject mixed valid and foreign filter values'
);

-- Should reject regions from another alliance
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            jsonb_build_object('regions', jsonb_build_array('none', %L))
        )$$,
        :'allianceID',
        :'foreignRegionID'
    ),
    'OCG01',
    'region not found',
    'Should reject regions from another alliance'
);

-- Should reject unknown group roles
select throws_ok(
    format(
        $$select validate_alliance_contact_filters(
            %L::uuid,
            '{"roles": ["admin", "organizer"]}'::jsonb
        )$$,
        :'allianceID'
    ),
    'OCG01',
    'group role not found',
    'Should reject unknown group roles'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
