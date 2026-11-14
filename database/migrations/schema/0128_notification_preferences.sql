-- Add category notification preferences and per-group notification mutes.

-- Catalog of notification categories users can turn off.
create table notification_category (
    notification_category_id text primary key,
    display_name text not null unique check (btrim(display_name) <> ''),
    group_mutable boolean not null
);

-- Seed the supported notification categories.
insert into notification_category (notification_category_id, display_name, group_mutable) values
    ('attendee-activity', 'Attendee activity', false),
    ('badges', 'Badges', true),
    ('cohosting-updates', 'Co-hosting updates', false),
    ('event-reminders', 'Event reminders', true),
    ('group-announcements', 'Group announcements', true),
    ('group-inbox', 'Group inbox', false),
    ('new-events', 'New events', true),
    ('opportunity-digests', 'Opportunity digests', false),
    ('organizer-messages', 'Messages from event organizers', true),
    ('paid-event-setups', 'Paid event setups', false);

-- Assign notification kinds to categories; kinds without one are always sent.
alter table notification_kind
    add column notification_category_id text references notification_category;

update notification_kind set notification_category_id = 'attendee-activity'
where name = 'event-admission-offer-declined';

update notification_kind set notification_category_id = 'badges'
where name in ('badge-awarded', 'badge-revoked');

update notification_kind set notification_category_id = 'cohosting-updates'
where name in ('event-cohost-removed', 'event-cohost-responded');

update notification_kind set notification_category_id = 'event-reminders'
where name = 'event-reminder';

update notification_kind set notification_category_id = 'group-announcements'
where name = 'group-custom';

update notification_kind set notification_category_id = 'group-inbox'
where name = 'inbox-message-received';

update notification_kind set notification_category_id = 'new-events'
where name in ('event-published', 'event-series-published');

update notification_kind set notification_category_id = 'opportunity-digests'
where name = 'opportunity-digest';

update notification_kind set notification_category_id = 'organizer-messages'
where name = 'event-custom';

update notification_kind set notification_category_id = 'paid-event-setups'
where name = 'event-paid-configured';

-- Categories each user has turned off.
create table user_notification_opt_out (
    created_at timestamptz default current_timestamp not null,
    notification_category_id text not null references notification_category,
    user_id uuid not null references "user" on delete cascade,

    primary key (user_id, notification_category_id)
);

-- Groups each user has muted.
create table user_group_notification_mute (
    created_at timestamptz default current_timestamp not null,
    group_id uuid not null references "group" on delete cascade,
    user_id uuid not null references "user" on delete cascade,

    primary key (user_id, group_id)
);

-- Support mute lookups when a group is deleted.
create index user_group_notification_mute_group_id_idx on user_group_notification_mute (group_id);

-- Keep the member categories off for users who had optional notifications off.
insert into user_notification_opt_out (notification_category_id, user_id)
select nc.notification_category_id, u.user_id
from "user" u
cross join notification_category nc
where u.optional_notifications_enabled = false
and nc.notification_category_id in (
    'event-reminders',
    'group-announcements',
    'new-events',
    'organizer-messages'
);

-- Remove the replaced optional notification flags.
alter table "user" drop column optional_notifications_enabled;
alter table notification_kind drop column optional_notification;

-- Replace the enqueue helper with one that accepts the groups a notification names.
drop function if exists enqueue_notification(text, jsonb, jsonb, uuid[]);

-- Replace the co-host recipient lookup after removing the legacy user-level
-- optional-notification flag. Co-host preferences now use the category opt-out.
create or replace function list_event_cohost_notification_recipients(p_event_id uuid)
returns table (cohost_group_id uuid, user_id uuid) as $$
    select distinct ec.cohost_group_id, member.user_id
    from event_cohost ec
    join "group" g on g.group_id = ec.cohost_group_id
    join lateral (
        select gm.user_id from group_member gm where gm.group_id = ec.cohost_group_id
        union
        select gt.user_id from group_team gt
        where gt.group_id = ec.cohost_group_id and gt.accepted = true
    ) member on true
    left join group_cohost_notification_preference pref
      on pref.group_id = ec.cohost_group_id and pref.user_id = member.user_id
    where ec.event_id = p_event_id
      and ec.status = 'approved'
      and g.active = true
      and g.deleted = false
      and coalesce(pref.enabled, true) = true
      and not exists (
          select 1
          from user_notification_opt_out uo
          where uo.user_id = member.user_id
            and uo.notification_category_id = 'cohosting-updates'
      );
$$ language sql stable;
