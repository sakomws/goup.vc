begin;
select plan(18);

\set allianceID '7d150000-0000-0000-0000-000000000001'
\set groupCategoryID '7d150000-0000-0000-0000-000000000002'
\set groupID '7d150000-0000-0000-0000-000000000003'
\set userID '7d150000-0000-0000-0000-000000000004'
\set eventManagerID '7d150000-0000-0000-0000-000000000007'
\set campaignID '7d150000-0000-0000-0000-000000000005'
\set partnerID '7d150000-0000-0000-0000-000000000006'
\set eventCategoryID '7d150000-0000-0000-0000-000000000008'
\set eventID '7d150000-0000-0000-0000-000000000009'

insert into alliance (alliance_id, name, display_name, description, banner_mobile_url, banner_url, logo_url)
values (:'allianceID', 'distribution-test', 'Distribution Test', 'Test', 'https://example.com/m.png', 'https://example.com/b.png', 'https://example.com/l.png');
insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Community');
insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Meetup');
insert into "user" (user_id, auth_hash, email, email_verified, username, name)
values
    (:'userID', gen_random_bytes(32), 'distribution@example.com', true, 'distribution-user', 'Distribution User'),
    (:'eventManagerID', gen_random_bytes(32), 'event-manager@example.com', true, 'distribution-events', 'Event Manager');
insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values (:'groupID', :'allianceID', :'groupCategoryID', 'Distribution Group', 'distribution-group');
insert into group_team (group_id, user_id, role, accepted)
values
    (:'groupID', :'userID', 'admin', true),
    (:'groupID', :'eventManagerID', 'events-manager', true);
insert into event (
    event_id, group_id, name, slug, description, timezone, event_category_id, event_kind_id
) values (
    :'eventID', :'groupID', 'Launch Event', 'launch-event', 'Launch', 'UTC',
    :'eventCategoryID', 'in-person'
);

insert into distribution_campaign (
    distribution_campaign_id, group_id, event_id, created_by, name, status
) values (:'campaignID', :'groupID', :'eventID', :'userID', 'Launch', 'active');
insert into distribution_partner (distribution_partner_id, group_id, name, referral_code)
values (:'partnerID', :'groupID', 'Partner', 'PARTNER');
insert into distribution_link (
    distribution_campaign_id, distribution_partner_id, code, channel, target_url,
    utm_source, utm_medium, utm_campaign, utm_content
) values (
    :'campaignID', :'partnerID', 'launch-1', 'linkedin', 'https://events.example/register?keep=1',
    'linkedin', 'social', 'launch', 'speaker'
);

select has_table('distribution_campaign');
select has_table('distribution_partner');
select has_table('distribution_link');
select has_table('distribution_link_click_daily');
select has_table('distribution_content');
select has_table('distribution_library_item');
select hasnt_column(
    'distribution_link_click_daily',
    'ip_address',
    'raw IP addresses are not stored'
);
select hasnt_column(
    'distribution_link_click_daily',
    'user_agent',
    'raw user agents are not stored'
);
select has_function('get_distribution_dashboard', array['uuid']);
select has_function('resolve_distribution_link', array['text', 'text']);
select ok(
    user_has_group_permission(:'allianceID', :'groupID', :'userID', 'group.distribution.write'),
    'group admins receive distribution write permission'
);
select isnt(
    user_has_group_permission(:'allianceID', :'groupID', :'eventManagerID', 'group.distribution.write'),
    true,
    'event managers do not receive distribution write permission'
);

select is(
    resolve_distribution_link('LAUNCH-1', repeat('a', 64))->>'utm_campaign',
    'launch',
    'tracked links resolve case-insensitively with frozen attribution'
);
select lives_ok(
    $$select resolve_distribution_link('launch-1', repeat('a', 64))$$,
    'repeated click recording is idempotent'
);
select is(
    (select count(*)::integer from distribution_link_click_daily),
    1,
    'daily click fingerprint is deduplicated'
);
do $$
begin
    if to_regclass('public.event_registration_attribution') is not null then
        execute format(
            'insert into event_registration_attribution (
                event_id, user_id, referral_code, utm_source, utm_medium, utm_campaign, utm_content
            ) values (%L, %L, %L, %L, %L, %L, %L)',
            '7d150000-0000-0000-0000-000000000009',
            '7d150000-0000-0000-0000-000000000004',
            'PARTNER', 'linkedin', 'social', 'launch', 'speaker'
        );
    end if;
end;
$$;
select ok(
    (get_distribution_dashboard(:'groupID'::uuid)->'campaigns'->0->>'registrations')::integer =
        case when to_regclass('public.event_registration_attribution') is null then 0 else 1 end,
    'dashboard integrates attribution when present and works when absent'
);

insert into distribution_content (
    distribution_campaign_id, channel, state, title, scheduled_for, remind_when_due
) values (
    :'campaignID', 'linkedin', 'ready', 'Due post', current_timestamp - interval '1 minute', true
);
select is(
    enqueue_due_distribution_content_reminders('https://example.test'),
    1,
    'due content reminder is enqueued once'
);
select is(
    enqueue_due_distribution_content_reminders('https://example.test'),
    0,
    'due content reminder enqueue is idempotent'
);

select * from finish();
rollback;
