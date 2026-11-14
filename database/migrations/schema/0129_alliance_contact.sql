-- Add alliance-scoped custom notifications sent to group teams.

-- Allow custom notifications to be scoped to an alliance.
alter table custom_notification
    add column alliance_id uuid references alliance (alliance_id) on delete cascade;

-- Require exactly one custom notification scope.
alter table custom_notification drop constraint custom_notification_check;
alter table custom_notification add constraint custom_notification_scope_check check (
    num_nonnulls(alliance_id, event_id, group_id) = 1
);

-- Support alliance custom notification lookups and cascades.
create index custom_notification_alliance_id_idx on custom_notification (alliance_id);

-- Register the alliance custom notification kind, which is always sent.
insert into notification_kind (name) values ('alliance-custom');

-- Drop signatures replaced by the alliance-aware custom notification functions.
drop function if exists enqueue_tracked_custom_notification(
    text, jsonb, jsonb, uuid[], uuid, uuid, uuid, int, text, text
);
drop function if exists track_custom_notification(uuid, uuid, uuid, int, text, text);
