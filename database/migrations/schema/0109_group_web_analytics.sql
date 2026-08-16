-- Optional organizer-owned GA4 measurement ID for public event pages.
alter table "group"
    add column if not exists web_analytics_measurement_id text;

alter table "group"
    drop constraint if exists group_web_analytics_measurement_id_check;

alter table "group"
    add constraint group_web_analytics_measurement_id_check
    check (
        web_analytics_measurement_id is null
        or web_analytics_measurement_id ~ '^G-[A-Z0-9]{6,32}$'
    );
