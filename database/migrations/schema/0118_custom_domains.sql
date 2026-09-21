-- Custom hostnames can point to either a group or an event. Hostnames are
-- stored in canonical form so uniqueness and request-time lookup agree.

create table if not exists custom_domain (
    custom_domain_id uuid default gen_random_uuid() not null,
    hostname text not null,
    group_id uuid references "group" (group_id) on delete cascade,
    event_id uuid references event (event_id) on delete cascade,
    verification_token text not null,
    verification_token_created_at timestamp with time zone default current_timestamp not null,
    verified_at timestamp with time zone,
    activated_at timestamp with time zone,
    created_at timestamp with time zone default current_timestamp not null,
    updated_at timestamp with time zone default current_timestamp not null,
    constraint custom_domain_pkey primary key (custom_domain_id),
    constraint custom_domain_exactly_one_target_chk check (
        (group_id is not null)::integer + (event_id is not null)::integer = 1
    ),
    constraint custom_domain_hostname_normalized_chk check (
        hostname = lower(rtrim(btrim(hostname), '.'))
    ),
    constraint custom_domain_hostname_valid_chk check (
        length(hostname) between 1 and 253
        and hostname ~ '^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$'
        and regexp_replace(hostname, '^.*\.', '') ~ '^(?:[a-z]+|xn--[a-z0-9-]+)$'
        and hostname <> 'goup.vc'
        and hostname !~ '\.goup\.vc$'
    ),
    constraint custom_domain_verification_token_chk check (
        btrim(verification_token) <> ''
    ),
    constraint custom_domain_activation_requires_verification_chk check (
        activated_at is null or verified_at is not null
    )
);

create unique index if not exists custom_domain_hostname_key
    on custom_domain (hostname);

create unique index if not exists custom_domain_group_id_key
    on custom_domain (group_id)
    where group_id is not null;

create unique index if not exists custom_domain_event_id_key
    on custom_domain (event_id)
    where event_id is not null;
