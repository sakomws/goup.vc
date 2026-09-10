-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set allianceID 'c1150000-0000-0000-0000-000000000001'

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
    'gtm-review',
    'GTM Review',
    'Alliance used for GTM draft review tests',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

-- ============================================================================
-- TESTS
-- ============================================================================

select is(
    (
        select (review_gtm_agent_draft(
            null::uuid,
            :'allianceID'::uuid,
            add_gtm_agent_draft(
                null::uuid,
                :'allianceID'::uuid,
                jsonb_build_object(
                    'agent_id', 'lead_generation',
                    'title', 'Suggested leads',
                    'body', 'Create these leads',
                    'suggested_stage', 'lead_generation',
                    'payload', jsonb_build_object(
                        'candidates', jsonb_build_array(
                            jsonb_build_object(
                                'name', 'Candidate One',
                                'kind', 'startup',
                                'source', 'discovery',
                                'website_url', 'https://candidate.example'
                            )
                        )
                    )
                )
            ),
            jsonb_build_object('status', 'approved')
        )->>'status')
    ),
    'approved',
    'Approving a lead-generation draft should succeed'
);

select is(
    (
        select count(*)::int
        from gtm_lead
        where alliance_id = :'allianceID'::uuid
          and name = 'Candidate One'
    ),
    1,
    'Approving a lead-generation draft should create the candidate lead'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
