alter table jobs_discovery_item
    add column if not exists review_status text not null default 'pending'
        check (review_status in ('pending', 'published', 'rejected')),
    add column if not exists candidate_url text not null default '',
    add column if not exists discovered_payload jsonb,
    add column if not exists reviewed_at timestamptz,
    add column if not exists reviewed_by uuid references "user"(user_id) on delete set null;

create index if not exists jobs_discovery_item_pending_idx
    on jobs_discovery_item (user_id, created_at desc)
    where review_status = 'pending';

alter table group_event_integration_item
    add column if not exists review_status text not null default 'pending'
        check (review_status in ('pending', 'published', 'rejected')),
    add column if not exists candidate_url text not null default '',
    add column if not exists discovered_payload jsonb,
    add column if not exists reviewed_at timestamptz,
    add column if not exists reviewed_by uuid references "user"(user_id) on delete set null;

create index if not exists group_event_integration_item_pending_idx
    on group_event_integration_item (group_id, created_at desc)
    where review_status = 'pending';
