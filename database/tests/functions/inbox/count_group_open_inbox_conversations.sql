-- Tests counting the open inbox conversations of a group.

-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID '1b110000-0000-0000-0000-000000000001'
\set groupCategoryID '1b110000-0000-0000-0000-000000000002'
\set groupID '1b110000-0000-0000-0000-000000000003'
\set otherGroupID '1b110000-0000-0000-0000-000000000004'
\set quietGroupID '1b110000-0000-0000-0000-000000000005'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Alliance
select fx_alliance(:'allianceID');

-- Group category
select fx_group_category(:'groupCategoryID', :'allianceID');

-- Group with conversations in every status
select fx_group(:'groupID', :'allianceID', :'groupCategoryID');

-- Another group with an open conversation
select fx_group(:'otherGroupID', :'allianceID', :'groupCategoryID');

-- Group without conversations
select fx_group(:'quietGroupID', :'allianceID', :'groupCategoryID');

-- Conversations of the group in every status
insert into inbox_conversation (group_id, inbox_conversation_status_id) values
    (:'groupID', 'answered'),
    (:'groupID', 'closed'),
    (:'groupID', 'open'),
    (:'groupID', 'open');

-- Open conversation of another group
insert into inbox_conversation (group_id, inbox_conversation_status_id)
values (:'otherGroupID', 'open');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should count only the open conversations of the group
select is(
    count_group_open_inbox_conversations(:'groupID'::uuid),
    2,
    'Should count only the open conversations of the group'
);

-- Should return zero for a group without open conversations
select is(
    count_group_open_inbox_conversations(:'quietGroupID'::uuid),
    0,
    'Should return zero for a group without open conversations'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
