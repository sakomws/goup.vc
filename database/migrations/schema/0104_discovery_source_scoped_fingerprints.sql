-- Keep rejected discovery history per source so replacing a source starts a
-- fresh review queue. Pending and published candidates remain unique per owner
-- to prevent duplicate drafts when sources overlap.

alter table jobs_discovery_item
    drop constraint if exists jobs_discovery_item_user_id_fingerprint_key;

create unique index if not exists jobs_discovery_item_user_source_fingerprint_key
    on jobs_discovery_item (user_id, source_url, fingerprint);

create unique index if not exists jobs_discovery_item_active_fingerprint_key
    on jobs_discovery_item (user_id, fingerprint)
    where review_status in ('pending', 'published');

alter table group_event_integration_item
    drop constraint if exists group_event_integration_item_group_id_fingerprint_key;

create unique index if not exists group_event_integration_item_group_source_fingerprint_key
    on group_event_integration_item (group_id, source_url, fingerprint);

create unique index if not exists group_event_integration_item_active_fingerprint_key
    on group_event_integration_item (group_id, fingerprint)
    where review_status in ('pending', 'published');
