-- Unified opportunity board. Jobs and calls for speakers remain in their
-- source tables and are projected at query time; only native opportunity kinds
-- are stored here.
create type opportunity_board_row as (
    source_id uuid,
    source_kind text,
    kind text,
    title text,
    organization_name text,
    summary text,
    description text,
    apply_url text,
    location text,
    remote boolean,
    members_only boolean,
    tags text[],
    published boolean,
    created_at timestamptz,
    closes_at timestamptz,
    posted_by_user_id uuid,
    poster_username text,
    poster_name text
);

create table opportunity (
    opportunity_id uuid primary key default gen_random_uuid(),
    posted_by_user_id uuid not null references "user" (user_id) on delete cascade,
    alliance_id uuid references alliance (alliance_id) on delete set null,
    group_id uuid references "group" (group_id) on delete set null,
    kind text not null check (kind in ('grant', 'funding', 'partnership', 'research')),
    title text not null check (btrim(title) <> ''),
    slug text not null unique check (btrim(slug) <> ''),
    organization_name text not null check (btrim(organization_name) <> ''),
    summary text not null check (btrim(summary) <> ''),
    description text not null check (btrim(description) <> ''),
    apply_url text not null check (btrim(apply_url) <> ''),
    location text check (location is null or btrim(location) <> ''),
    remote boolean not null default false,
    members_only boolean not null default false,
    tags text[] not null default '{}',
    metadata jsonb not null default '{}',
    opens_at timestamptz,
    closes_at timestamptz,
    published boolean not null default false,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz,
    check (jsonb_typeof(metadata) = 'object'),
    check (closes_at is null or opens_at is null or closes_at >= opens_at)
);

create index opportunity_public_idx
    on opportunity (published, closes_at, created_at desc);
create index opportunity_owner_idx
    on opportunity (posted_by_user_id, created_at desc);
create index opportunity_kind_idx
    on opportunity (kind, published, created_at desc);
create index opportunity_tags_idx on opportunity using gin (tags);

create table opportunity_saved_search (
    opportunity_saved_search_id uuid primary key default gen_random_uuid(),
    user_id uuid not null references "user" (user_id) on delete cascade,
    name text not null check (btrim(name) <> ''),
    filters jsonb not null default '{}',
    frequency text not null check (frequency in ('daily', 'weekly')),
    active boolean not null default false,
    next_run_at timestamptz,
    last_run_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz,
    check (jsonb_typeof(filters) = 'object'),
    check ((active and next_run_at is not null) or not active),
    unique (user_id, name)
);

create index opportunity_saved_search_due_idx
    on opportunity_saved_search (next_run_at, opportunity_saved_search_id)
    where active;

-- A run is inserted before its notification is enqueued. The primary key is
-- the idempotency boundary for worker retries and concurrent workers.
create table opportunity_digest_run (
    opportunity_saved_search_id uuid not null references opportunity_saved_search
        (opportunity_saved_search_id) on delete cascade,
    scheduled_for timestamptz not null,
    match_count integer not null check (match_count >= 0),
    enqueued_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    primary key (opportunity_saved_search_id, scheduled_for)
);

insert into notification_kind (notification_kind_id, name, optional_notification)
values ('238d50b2-c5a9-4af3-93c6-75779313962c', 'opportunity-digest', true)
on conflict (name) do update
set optional_notification = excluded.optional_notification;
