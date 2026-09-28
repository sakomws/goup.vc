-- Planning-only distribution workspace. Social publishing remains manual.

create table if not exists distribution_campaign (
    distribution_campaign_id uuid primary key default gen_random_uuid(),
    group_id uuid not null references "group" (group_id) on delete cascade,
    event_id uuid references event (event_id) on delete set null,
    created_by uuid references "user" (user_id) on delete set null,
    name text not null check (btrim(name) <> ''),
    status text not null default 'draft' check (status in ('draft', 'active', 'completed')),
    starts_at timestamptz,
    ends_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp,
    check (starts_at is null or ends_at is null or starts_at <= ends_at)
);

create index if not exists distribution_campaign_group_idx
    on distribution_campaign (group_id, created_at desc);

create table if not exists distribution_partner (
    distribution_partner_id uuid primary key default gen_random_uuid(),
    group_id uuid not null references "group" (group_id) on delete cascade,
    name text not null check (btrim(name) <> ''),
    referral_code text not null check (referral_code ~ '^[A-Za-z0-9_-]{2,64}$'),
    active boolean not null default true,
    created_at timestamptz not null default current_timestamp
);

create unique index if not exists distribution_partner_group_code_key
    on distribution_partner (group_id, lower(referral_code));

create table if not exists distribution_link (
    distribution_link_id uuid primary key default gen_random_uuid(),
    distribution_campaign_id uuid not null references distribution_campaign (distribution_campaign_id) on delete cascade,
    distribution_partner_id uuid references distribution_partner (distribution_partner_id) on delete set null,
    code text not null check (code ~ '^[A-Za-z0-9_-]{3,64}$'),
    channel text not null check (channel in ('linkedin', 'x', 'instagram', 'email', 'partner', 'other')),
    target_url text not null check (target_url ~ '^https?://'),
    utm_source text not null check (btrim(utm_source) <> ''),
    utm_medium text not null check (btrim(utm_medium) <> ''),
    utm_campaign text not null check (btrim(utm_campaign) <> ''),
    utm_content text,
    frozen_at timestamptz not null default current_timestamp,
    active boolean not null default true,
    created_at timestamptz not null default current_timestamp
);

create unique index if not exists distribution_link_code_key
    on distribution_link (lower(code));

-- Stores only a one-way, day-scoped fingerprint. Raw address/header values are never persisted.
create table if not exists distribution_link_click_daily (
    distribution_link_id uuid not null references distribution_link (distribution_link_id) on delete cascade,
    clicked_on date not null default ((current_timestamp at time zone 'UTC')::date),
    fingerprint_hash text not null check (fingerprint_hash ~ '^[0-9a-f]{64}$'),
    created_at timestamptz not null default current_timestamp,
    primary key (distribution_link_id, clicked_on, fingerprint_hash)
);

create table if not exists distribution_content (
    distribution_content_id uuid primary key default gen_random_uuid(),
    distribution_campaign_id uuid not null references distribution_campaign (distribution_campaign_id) on delete cascade,
    channel text not null check (channel in ('linkedin', 'x', 'instagram')),
    state text not null default 'idea' check (state in ('idea', 'draft', 'ready', 'posted_manual')),
    title text not null check (btrim(title) <> ''),
    caption text not null default '',
    cta text not null default '',
    hashtags text not null default '',
    event_image_reference text,
    scheduled_for timestamptz,
    remind_when_due boolean not null default false,
    reminder_sent_for timestamptz,
    posted_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp,
    check ((state = 'posted_manual') = (posted_at is not null))
);

create index if not exists distribution_content_due_idx
    on distribution_content (scheduled_for)
    where state = 'ready' and remind_when_due and reminder_sent_for is null;

create table if not exists distribution_library_item (
    distribution_library_item_id uuid primary key default gen_random_uuid(),
    group_id uuid not null references "group" (group_id) on delete cascade,
    kind text not null check (kind in ('caption', 'cta', 'hashtags', 'event_image')),
    name text not null check (btrim(name) <> ''),
    value text not null check (btrim(value) <> ''),
    created_at timestamptz not null default current_timestamp
);

insert into group_permission (group_permission_id, display_name)
values ('group.distribution.write', 'Distribution Write')
on conflict (group_permission_id) do nothing;

insert into group_role_group_permission (group_permission_id, group_role_id)
values ('group.distribution.write', 'admin')
on conflict do nothing;

insert into alliance_role_group_permission (alliance_role_id, group_permission_id)
values ('admin', 'group.distribution.write'), ('groups-manager', 'group.distribution.write')
on conflict do nothing;

insert into notification_kind (name, optional_notification)
values ('distribution-content-due', true)
on conflict (name) do update set optional_notification = true;
