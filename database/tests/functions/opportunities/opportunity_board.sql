begin;
select plan(47);

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

select throws_like(
    $$insert into opportunity (
        posted_by_user_id, kind, title, slug, organization_name,
        summary, description, apply_url
    ) values (
        'b9230000-0000-0000-0000-000000000001',
        'grant', 'Unsafe grant', 'opp-unsafe', 'GOUP',
        'Unsafe link.', 'Unsafe link.', 'javascript:alert(1)'
    )$$,
    '%opportunity_apply_url_check%',
    'native application URLs reject unsafe schemes'
);

select lives_ok(
    $$insert into opportunity (
        opportunity_id, posted_by_user_id, kind, title, slug,
        organization_name, summary, description, apply_url,
        published, opens_at
    ) values (
        'b9230000-0000-0000-0000-000000000006',
        'b9230000-0000-0000-0000-000000000001',
        'grant', 'Future grant', 'opp1237', 'GOUP',
        'Not open yet.', 'Future details.',
        'https://example.test/future', true,
        current_timestamp + interval '1 day'
    )$$,
    'a scheduled native opportunity can be created'
);
select is(
    get_opportunity(
        'native', 'b9230000-0000-0000-0000-000000000006', false
    ),
    null,
    'native opportunities remain private until opens_at'
);

select lives_ok(
    $$insert into jobs_job (
        job_id, posted_by_user_id, title, slug, company_name,
        summary, description, apply_url, tags
    ) values (
        'b9230000-0000-0000-0000-000000000007',
        'b9230000-0000-0000-0000-000000000001',
        'Rust engineer', 'rust-engineer', 'GOUP',
        'Build community software.', 'Full job details.',
        'https://example.test/jobs/rust', array['rust']
    )$$,
    'a published job projection can be created'
);
select is(
    get_opportunity(
        'job', 'b9230000-0000-0000-0000-000000000007', false
    )->>'title',
    'Rust engineer',
    'published jobs are projected into the opportunity board'
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
    $$update "user"
      set optional_notifications_enabled = true
      where user_id = 'b9230000-0000-0000-0000-000000000001'$$,
    'the owner can opt in to optional notifications'
);
select lives_ok(
    $$insert into opportunity (
        opportunity_id, posted_by_user_id, kind, title, slug,
        organization_name, summary, description, apply_url, tags,
        published, created_at
    ) values (
        'b9230000-0000-0000-0000-000000000008',
        'b9230000-0000-0000-0000-000000000001',
        'research', 'Compiler research', 'opp1238', 'GOUP',
        'Research opportunity.', 'Research details.',
        'https://example.test/research', array['compilers'], true,
        current_timestamp - interval '2 minutes'
    )$$,
    'a digest match can be created before the safe watermark'
);
select lives_ok(
    $$insert into opportunity_saved_search (
        opportunity_saved_search_id, user_id, name, filters,
        frequency, active, next_run_at, created_at
    ) values (
        'b9230000-0000-0000-0000-000000000009',
        'b9230000-0000-0000-0000-000000000001',
        'Compiler tags', '{"query":"compilers"}',
        'daily', true, current_timestamp - interval '2 minutes',
        current_timestamp - interval '3 minutes'
    )$$,
    'a tag-filtered digest can become due'
);
select is(
    enqueue_due_opportunity_digests('https://example.test'),
    1,
    'tag matches enqueue an opted-in digest'
);
select is(
    (select count(*)::int from notification
     where kind = 'opportunity-digest'
       and user_id = 'b9230000-0000-0000-0000-000000000001'),
    1,
    'one digest notification is created'
);
select lives_ok(
    $$update opportunity_saved_search
      set next_run_at = current_timestamp - interval '2 minutes',
          last_run_at = null
      where opportunity_saved_search_id =
          'b9230000-0000-0000-0000-000000000009'$$,
    'the same scheduled run can be retried'
);
select is(
    enqueue_due_opportunity_digests('https://example.test'),
    0,
    'a retried schedule does not enqueue twice'
);
select is(
    (select count(*)::int from notification
     where kind = 'opportunity-digest'
       and user_id = 'b9230000-0000-0000-0000-000000000001'),
    1,
    'digest idempotency leaves one notification'
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
    '3',
    'anonymous search excludes member-only and expired records'
);
select is(
    search_opportunities('{"include_members_only":true}'::jsonb)->>'total',
    '4',
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
