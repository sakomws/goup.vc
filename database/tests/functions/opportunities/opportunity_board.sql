begin;
select plan(34);

select has_table('opportunity');
select has_table('opportunity_saved_search');
select has_table('opportunity_digest_run');
select col_is_pk('opportunity', 'opportunity_id');
select col_is_pk('opportunity_saved_search', 'opportunity_saved_search_id');
select has_check('opportunity');
select has_check('opportunity_saved_search');
select has_index('opportunity', 'opportunity_public_idx');
select has_index('opportunity_saved_search', 'opportunity_saved_search_due_idx');
select has_function('opportunity_rows', array['boolean']::name[]);
select has_function('search_opportunities', array['jsonb']::name[]);
select has_function('get_opportunity', array['text', 'uuid', 'boolean']::name[]);
select has_function('list_user_opportunities', array['uuid', 'jsonb']::name[]);
select has_function('add_opportunity', array['uuid', 'jsonb', 'text[]']::name[]);
select has_function('update_opportunity', array['uuid', 'uuid', 'jsonb', 'text[]']::name[]);
select has_function('update_opportunity_published', array['uuid', 'uuid', 'boolean']::name[]);
select has_function('upsert_opportunity_saved_search', array['uuid', 'uuid', 'jsonb']::name[]);
select has_function('activate_opportunity_saved_search', array['uuid', 'uuid']::name[]);
select has_function('delete_opportunity_saved_search', array['uuid', 'uuid']::name[]);
select has_function('enqueue_due_opportunity_digests', array['text']::name[]);

select lives_ok(
    $$insert into "user" (
        user_id, auth_hash, email, email_verified, username,
        optional_notifications_enabled
    ) values (
        'b9230000-0000-0000-0000-000000000001', 'hash',
        'opportunity-owner@example.com', true, 'opportunity-owner', false
    )$$,
    'owner can be created for opportunity function tests'
);

select lives_ok(
    $$insert into opportunity (
        opportunity_id, posted_by_user_id, kind, title, slug,
        organization_name, summary, description, apply_url, published
    ) values (
        'b9230000-0000-0000-0000-000000000002',
        'b9230000-0000-0000-0000-000000000001',
        'grant', 'Open source grant', 'opp1234', 'GOUP',
        'Fund useful open source work.', 'Full grant details.',
        'https://example.test/apply', true
    )$$,
    'native opportunity can be created'
);

select is(
    search_opportunities('{"query":"open source"}'::jsonb)->>'total',
    '1',
    'search returns matching native opportunities'
);
select is(
    get_opportunity(
        'native', 'b9230000-0000-0000-0000-000000000002', false
    )->>'title',
    'Open source grant',
    'details returns a visible native opportunity'
);
select throws_ok(
    $$select update_opportunity_published(
        'b9230000-0000-0000-0000-000000000099',
        'b9230000-0000-0000-0000-000000000002', false
    )$$,
    'opportunity not found',
    'non-owners cannot moderate native opportunities'
);

select lives_ok(
    $$insert into opportunity_saved_search (
        opportunity_saved_search_id, user_id, name, filters,
        frequency, active, next_run_at
    ) values (
        'b9230000-0000-0000-0000-000000000003',
        'b9230000-0000-0000-0000-000000000001',
        'Open source grants', '{"kind":"grant"}', 'daily', true,
        current_timestamp - interval '1 minute'
    )$$,
    'an active saved search can become due'
);
select is(
    enqueue_due_opportunity_digests('https://example.test'),
    0,
    'optional-notification opt-out suppresses enqueue'
);
select is(
    (select count(*)::int from opportunity_digest_run
     where opportunity_saved_search_id =
        'b9230000-0000-0000-0000-000000000003'),
    1,
    'a digest run is durably recorded'
);
select is(
    enqueue_due_opportunity_digests('https://example.test'),
    0,
    'immediate retry does not enqueue a duplicate digest'
);
select is(
    (select count(*)::int from opportunity_digest_run
     where opportunity_saved_search_id =
        'b9230000-0000-0000-0000-000000000003'),
    1,
    'idempotent retry preserves one digest run'
);

select lives_ok(
    $$insert into opportunity (
        opportunity_id, posted_by_user_id, kind, title, slug,
        organization_name, summary, description, apply_url,
        members_only, published, closes_at
    ) values
    (
        'b9230000-0000-0000-0000-000000000004',
        'b9230000-0000-0000-0000-000000000001',
        'research', 'Member research call', 'opp1235', 'GOUP',
        'Private member call.', 'Private details.',
        'https://example.test/member', true, true, null
    ),
    (
        'b9230000-0000-0000-0000-000000000005',
        'b9230000-0000-0000-0000-000000000001',
        'funding', 'Expired funding', 'opp1236', 'GOUP',
        'Closed call.', 'Closed details.',
        'https://example.test/closed', false, true,
        current_timestamp - interval '1 day'
    )$$,
    'member-only and expired visibility fixtures can be created'
);
select is(
    search_opportunities('{}'::jsonb)->>'total',
    '1',
    'anonymous search excludes member-only and expired records'
);
select is(
    search_opportunities('{"include_members_only":true}'::jsonb)->>'total',
    '2',
    'member search includes member-only but not expired records'
);
select is(
    get_opportunity(
        'native', 'b9230000-0000-0000-0000-000000000004', false
    ),
    null,
    'anonymous details cannot expose a member-only record'
);

select * from finish();
rollback;
