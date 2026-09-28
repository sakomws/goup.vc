-- Co-host invitation notifications are optional organizer mail. Delivery claims
-- share the existing co-host ledger so retries cannot enqueue duplicate mail.
insert into notification_kind (notification_kind_id, name, optional_notification)
values ('52f0a7d9-9f41-4b0c-9fa3-f3fa9d1ed2a5', 'event-cohost-invitation', true)
on conflict (name) do update
set optional_notification = excluded.optional_notification;

alter table event_cohost_delivery
    drop constraint event_cohost_delivery_delivery_kind_check;
alter table event_cohost_delivery
    add constraint event_cohost_delivery_delivery_kind_check
    check (delivery_kind in (
        'event-published',
        'calendar-invite',
        'event-rescheduled',
        'event-canceled',
        'cohost-invitation'
    ));
