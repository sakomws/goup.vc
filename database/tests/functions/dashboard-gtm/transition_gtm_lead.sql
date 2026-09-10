-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID 'b1150000-0000-0000-0000-000000000001'

-- ============================================================================
-- SEED DATA
-- ============================================================================

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
    'gtm-transition',
    'GTM Transition',
    'Alliance used for GTM transition tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

-- ============================================================================
-- TESTS
-- ============================================================================

select lives_ok(
    format(
        $sql$
        select transition_gtm_lead(
            null::uuid,
            %L::uuid,
            add_gtm_lead(null::uuid, %L::uuid, jsonb_build_object(
                'name', 'Reachable Lead',
                'kind', 'speaker'
            )),
            'reachout',
            true,
            '{}'::jsonb
        )
        $sql$,
        :'allianceID',
        :'allianceID'
    ),
    'Humans can move a new lead to reachout'
);

select throws_ok(
    format(
        $sql$
        select transition_gtm_lead(
            null::uuid,
            %L::uuid,
            add_gtm_lead(null::uuid, %L::uuid, jsonb_build_object(
                'name', 'Agent Locked Lead',
                'kind', 'investor'
            )),
            'won',
            false,
            '{}'::jsonb
        )
        $sql$,
        :'allianceID',
        :'allianceID'
    ),
    'P0001',
    'illegal gtm stage transition from lead_generation to won',
    'Agents cannot skip to won'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
