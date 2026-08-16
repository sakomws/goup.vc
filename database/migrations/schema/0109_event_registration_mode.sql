-- Registration ownership for events. Existing events continue using Goups's
-- built-in flow until organizers explicitly select another mode.
alter table event
    add column if not exists registration_mode text not null default 'built_in',
    add column if not exists external_registration_url text;

alter table event
    drop constraint if exists event_registration_mode_chk,
    add constraint event_registration_mode_chk
        check (registration_mode in ('built_in', 'external', 'none')),
    drop constraint if exists event_external_registration_url_chk,
    add constraint event_external_registration_url_chk
        check (
            (registration_mode = 'external' and nullif(btrim(external_registration_url), '') is not null)
            or (registration_mode <> 'external' and external_registration_url is null)
        );
