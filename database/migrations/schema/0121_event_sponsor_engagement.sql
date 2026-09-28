-- Aggregate, privacy-preserving event sponsor engagement and shareable reports.

create table event_sponsor_engagement_daily (
    event_id uuid not null,
    group_sponsor_id uuid not null,
    day date not null default current_date,
    metric text not null check (metric in ('impression', 'click')),
    visitor_hash bytea not null,
    created_at timestamptz not null default current_timestamp,
    primary key (event_id, group_sponsor_id, day, metric, visitor_hash),
    foreign key (group_sponsor_id, event_id)
        references event_sponsor (group_sponsor_id, event_id) on delete cascade
);

create index event_sponsor_engagement_daily_rollup_idx
    on event_sponsor_engagement_daily (event_id, group_sponsor_id, day, metric);

create table event_sponsor_manual_engagement (
    event_id uuid not null,
    group_sponsor_id uuid not null,
    leads_count integer not null default 0 check (leads_count >= 0),
    conversations_count integer not null default 0 check (conversations_count >= 0),
    meetings_count integer not null default 0 check (meetings_count >= 0),
    notes text,
    updated_by uuid references "user" (user_id) on delete set null,
    updated_at timestamptz not null default current_timestamp,
    primary key (event_id, group_sponsor_id),
    foreign key (group_sponsor_id, event_id)
        references event_sponsor (group_sponsor_id, event_id) on delete cascade,
    check (notes is null or length(notes) <= 2000)
);

create table event_sponsor_report_share (
    event_id uuid primary key references event (event_id) on delete cascade,
    token uuid not null unique default gen_random_uuid(),
    created_by uuid references "user" (user_id) on delete set null,
    created_at timestamptz not null default current_timestamp,
    revoked_at timestamptz
);

create index event_sponsor_report_share_active_token_idx
    on event_sponsor_report_share (token) where revoked_at is null;
