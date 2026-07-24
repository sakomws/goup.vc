begin;
select plan(8);

insert into "user" (user_id, auth_hash, email, username)
values (
    '10000000-0000-0000-0000-000000000001',
    'discovery-fingerprint-test',
    'discovery-fingerprint-test@example.com',
    'discovery-fingerprint-test'
);

insert into alliance (
    alliance_id, name, display_name, description, logo_url, banner_url, banner_mobile_url
) values (
    '30000000-0000-0000-0000-000000000001',
    'discovery-fingerprint-test',
    'Discovery fingerprint test',
    'Discovery fingerprint test alliance',
    'https://example.com/logo.png',
    'https://example.com/banner.png',
    'https://example.com/banner-mobile.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (
    '40000000-0000-0000-0000-000000000001',
    '30000000-0000-0000-0000-000000000001',
    'Discovery fingerprint test'
);

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values (
    '20000000-0000-0000-0000-000000000001',
    '30000000-0000-0000-0000-000000000001',
    '40000000-0000-0000-0000-000000000001',
    'Discovery fingerprint test',
    'discovery-fingerprint-test'
);

select lives_ok(
    $$insert into jobs_discovery_item (user_id, source_url, fingerprint, review_status)
      values ('10000000-0000-0000-0000-000000000001', 'https://old.example/jobs', 'job-rejected', 'rejected')$$,
    'a rejected job candidate is retained'
);
select lives_ok(
    $$insert into jobs_discovery_item (user_id, source_url, fingerprint)
      values ('10000000-0000-0000-0000-000000000001', 'https://replacement.example/jobs', 'job-rejected')$$,
    'a replacement job source can queue a previously rejected candidate'
);
select throws_ok(
    $$insert into jobs_discovery_item (user_id, source_url, fingerprint)
      values ('10000000-0000-0000-0000-000000000001', 'https://replacement.example/jobs', 'job-rejected')$$,
    '23505',
    null,
    'the same job source cannot queue a candidate twice'
);
select throws_ok(
    $$insert into jobs_discovery_item (user_id, source_url, fingerprint)
      values ('10000000-0000-0000-0000-000000000001', 'https://another.example/jobs', 'job-rejected')$$,
    '23505',
    null,
    'an active job candidate cannot create a duplicate draft from another source'
);

select lives_ok(
    $$insert into group_event_integration_item (group_id, source_url, fingerprint, review_status)
      values ('20000000-0000-0000-0000-000000000001', 'https://old.example/events', 'event-rejected', 'rejected')$$,
    'a rejected event candidate is retained'
);
select lives_ok(
    $$insert into group_event_integration_item (group_id, source_url, fingerprint)
      values ('20000000-0000-0000-0000-000000000001', 'https://replacement.example/events', 'event-rejected')$$,
    'a replacement event source can queue a previously rejected candidate'
);
select throws_ok(
    $$insert into group_event_integration_item (group_id, source_url, fingerprint)
      values ('20000000-0000-0000-0000-000000000001', 'https://replacement.example/events', 'event-rejected')$$,
    '23505',
    null,
    'the same event source cannot queue a candidate twice'
);
select throws_ok(
    $$insert into group_event_integration_item (group_id, source_url, fingerprint)
      values ('20000000-0000-0000-0000-000000000001', 'https://another.example/events', 'event-rejected')$$,
    '23505',
    null,
    'an active event candidate cannot create a duplicate draft from another source'
);

select * from finish();
rollback;
