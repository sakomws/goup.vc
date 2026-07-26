-- Rejected candidates may be discovered again from the same source. Only
-- pending and published candidates reserve a source/fingerprint combination.

drop index if exists jobs_discovery_item_user_source_fingerprint_key;
create unique index if not exists jobs_discovery_item_active_source_fingerprint_key
    on jobs_discovery_item (user_id, source_url, fingerprint)
    where review_status in ('pending', 'published');

drop index if exists group_event_integration_item_group_source_fingerprint_key;
create unique index if not exists group_event_integration_item_active_source_fingerprint_key
    on group_event_integration_item (group_id, source_url, fingerprint)
    where review_status in ('pending', 'published');
