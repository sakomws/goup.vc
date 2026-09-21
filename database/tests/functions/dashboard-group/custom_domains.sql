begin;
select plan(30);

\set allianceID '7c440000-0000-0000-0000-000000000001'
\set eventCategoryID '7c440000-0000-0000-0000-000000000002'
\set eventID '7c440000-0000-0000-0000-000000000003'
\set groupAdminID '7c440000-0000-0000-0000-000000000004'
\set groupCategoryID '7c440000-0000-0000-0000-000000000005'
\set groupID '7c440000-0000-0000-0000-000000000006'
\set eventManagerID '7c440000-0000-0000-0000-000000000007'

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
    'custom-domain-alliance',
    'Custom Domain Alliance',
    'Tests custom domains',
    'https://example.com/banner-mobile.png',
    'https://example.com/banner.png',
    'https://example.com/logo.png'
);

insert into group_category (group_category_id, alliance_id, name)
values (:'groupCategoryID', :'allianceID', 'Technology');

insert into event_category (event_category_id, alliance_id, name)
values (:'eventCategoryID', :'allianceID', 'Meetup');

insert into "user" (user_id, auth_hash, email, email_verified, username, name) values
    (:'groupAdminID', gen_random_bytes(32), 'domain-admin@example.com', true, 'domain-admin', 'Domain Admin'),
    (:'eventManagerID', gen_random_bytes(32), 'domain-events@example.com', true, 'domain-events', 'Event Manager');

insert into "group" (group_id, alliance_id, group_category_id, name, slug)
values (:'groupID', :'allianceID', :'groupCategoryID', 'Domain Group', 'domain-group');

insert into group_team (group_id, user_id, role, accepted) values
    (:'groupID', :'groupAdminID', 'admin', true),
    (:'groupID', :'eventManagerID', 'events-manager', true);

insert into event (
    event_id,
    group_id,
    name,
    slug,
    description,
    timezone,
    event_category_id,
    event_kind_id
) values (
    :'eventID',
    :'groupID',
    'Domain Event',
    'domain-event',
    'Custom domain event',
    'UTC',
    :'eventCategoryID',
    'in-person'
);

select is(
    normalize_custom_domain_hostname('  Events.Example.COM.  '),
    'events.example.com',
    'Hostname normalization lowercases, trims whitespace, and removes a trailing dot'
);

select is(
    upsert_custom_domain(
        :'groupAdminID'::uuid,
        :'allianceID'::uuid,
        :'groupID'::uuid,
        null,
        '  Community.Example.COM. ',
        'goup-verification=group-token-1'
    )->>'hostname',
    'community.example.com',
    'Group admins can create a normalized group domain'
);

select ok(
    exists (
        select 1
        from custom_domain
        where group_id = :'groupID'::uuid
          and event_id is null
    ),
    'A group domain has exactly the group target'
);

select is(
    get_custom_domain(:'groupID'::uuid, null)->>'hostname',
    'community.example.com',
    'Authorized users can fetch a group domain'
);

select throws_ok(
    format(
        $$select upsert_custom_domain(%L::uuid, %L::uuid, %L::uuid, null, 'forbidden.example.com', 'forbidden-token')$$,
        :'eventManagerID',
        :'allianceID',
        :'groupID'
    ),
    'custom domain permission required',
    'Event managers cannot manage a group domain'
);

select is(
    upsert_custom_domain(
        :'eventManagerID'::uuid,
        :'allianceID'::uuid,
        :'groupID'::uuid,
        :'eventID'::uuid,
        'EVENT.EXAMPLE.COM',
        'goup-verification=event-token'
    )->>'hostname',
    'event.example.com',
    'Event managers can create an event domain'
);

select ok(
    exists (
        select 1
        from custom_domain
        where event_id = :'eventID'::uuid
          and group_id is null
    ),
    'An event domain has exactly the event target'
);

select throws_ok(
    format(
        $$select upsert_custom_domain(%L::uuid, %L::uuid, %L::uuid, %L::uuid, 'community.example.com', 'duplicate-token')$$,
        :'eventManagerID',
        :'allianceID',
        :'groupID',
        :'eventID'
    ),
    'hostname is already assigned',
    'Normalized hostnames are globally unique'
);

select throws_ok(
    format(
        $$select upsert_custom_domain(%L::uuid, %L::uuid, %L::uuid, null, 'https://bad.example.com/path', 'invalid-host-token')$$,
        :'groupAdminID',
        :'allianceID',
        :'groupID'
    ),
    'hostname is invalid',
    'URLs are rejected when a hostname is required'
);

select throws_ok(
    format(
        $$select upsert_custom_domain(%L::uuid, %L::uuid, %L::uuid, null, 'events.goup.vc', 'reserved-host-token')$$,
        :'groupAdminID',
        :'allianceID',
        :'groupID'
    ),
    'hostname is invalid',
    'GOUP-owned hostnames are reserved'
);

select throws_ok(
    $$insert into custom_domain (hostname, verification_token) values ('none.example.com', 'none-token')$$,
    '23514',
    null,
    'A custom domain cannot have no target'
);

select throws_ok(
    format(
        $$insert into custom_domain (hostname, group_id, event_id, verification_token) values ('both.example.com', %L::uuid, %L::uuid, 'both-token')$$,
        :'groupID',
        :'eventID'
    ),
    '23514',
    null,
    'A custom domain cannot have both targets'
);

select custom_domain_id, verification_token
from custom_domain
where group_id = :'groupID'::uuid
\gset old_

select is(
    upsert_custom_domain(
        :'groupAdminID'::uuid,
        :'allianceID'::uuid,
        :'groupID'::uuid,
        null,
        'new.example.com',
        'goup-verification=group-token-2'
    )->>'hostname',
    'new.example.com',
    'Replacing a target updates its hostname'
);

select custom_domain_id, verification_token, verified_at, activated_at
from custom_domain
where group_id = :'groupID'::uuid
\gset current_

select is(
    :'current_custom_domain_id'::uuid,
    :'old_custom_domain_id'::uuid,
    'Replacing preserves the custom domain identifier'
);

select isnt(
    :'current_verification_token'::text,
    :'old_verification_token'::text,
    'Replacing rotates the verification token'
);

select is(
    (select verified_at from custom_domain where group_id = :'groupID'::uuid),
    null::timestamp with time zone,
    'Replacing clears verification'
);

select is(
    (select activated_at from custom_domain where group_id = :'groupID'::uuid),
    null::timestamp with time zone,
    'Replacing clears activation'
);

select is(
    mark_custom_domain_active(:'current_custom_domain_id'::uuid),
    false,
    'An unverified domain cannot be activated'
);

select throws_ok(
    format(
        $$select mark_custom_domain_verified(%L::uuid, %L::uuid, null, %L::uuid, 'new.example.com', 'goup-verification=group-token-2')$$,
        :'eventManagerID',
        :'groupID',
        :'current_custom_domain_id'
    ),
    'custom domain permission required',
    'Verification enforces target-specific permissions'
);

select is(
    mark_custom_domain_verified(
        :'groupAdminID'::uuid,
        :'groupID'::uuid,
        null,
        :'current_custom_domain_id'::uuid,
        'new.example.com',
        'goup-verification=group-token-2'
    )->>'hostname',
    'new.example.com',
    'Authorized verification marks the current domain verified'
);

select isnt(
    (select verified_at from custom_domain where group_id = :'groupID'::uuid),
    null::timestamp with time zone,
    'Verification stores its timestamp'
);

select is(
    mark_custom_domain_active(:'current_custom_domain_id'::uuid),
    true,
    'A verified domain can be activated'
);

select is(
    resolve_active_custom_domain(' NEW.EXAMPLE.COM. '),
    jsonb_build_object(
        'alliance_name', 'custom-domain-alliance',
        'group_slug', 'domain-group',
        'hostname', 'new.example.com'
    ),
    'Active hostname resolution normalizes input and returns its target'
);

select is(
    resolve_active_custom_domain('event.example.com'),
    null::jsonb,
    'An unactivated hostname does not resolve'
);

select is(
    upsert_custom_domain(
        :'groupAdminID'::uuid,
        :'allianceID'::uuid,
        :'groupID'::uuid,
        null,
        'replacement.example.com',
        'goup-verification=group-token-3'
    )->>'hostname',
    'replacement.example.com',
    'An active domain can be replaced'
);

select is(
    resolve_active_custom_domain('new.example.com'),
    null::jsonb,
    'A replaced hostname stops resolving'
);

select is(
    delete_custom_domain(
        :'eventManagerID'::uuid,
        :'groupID'::uuid,
        :'eventID'::uuid
    ),
    true,
    'Event managers can delete an event domain'
);

select is(
    get_custom_domain(:'groupID'::uuid, :'eventID'::uuid),
    null::jsonb,
    'Fetching a deleted domain returns null'
);

select is(
    delete_custom_domain(
        :'eventManagerID'::uuid,
        :'groupID'::uuid,
        :'eventID'::uuid
    ),
    false,
    'Deleting a missing domain reports false'
);

select is(
    (select count(*) from custom_domain where group_id = :'groupID'::uuid),
    1::bigint,
    'Only one custom domain exists for a group'
);

select * from finish();
rollback;
