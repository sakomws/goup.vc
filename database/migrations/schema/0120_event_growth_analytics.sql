-- Event growth attribution, collaboration follow-up, and explicit manual finances.

create table if not exists event_registration_attribution (
    event_id uuid not null references event (event_id) on delete cascade,
    user_id uuid not null references "user" (user_id) on delete cascade,
    source text,
    referral_code text,
    referrer text,
    utm_source text,
    utm_medium text,
    utm_campaign text,
    utm_content text,
    utm_term text,
    captured_at timestamptz not null default current_timestamp,
    primary key (event_id, user_id),
    check (source is null or btrim(source) <> ''),
    check (referral_code is null or btrim(referral_code) <> ''),
    check (referrer is null or btrim(referrer) <> ''),
    check (utm_source is null or btrim(utm_source) <> ''),
    check (utm_medium is null or btrim(utm_medium) <> ''),
    check (utm_campaign is null or btrim(utm_campaign) <> ''),
    check (utm_content is null or btrim(utm_content) <> ''),
    check (utm_term is null or btrim(utm_term) <> '')
);

create index if not exists event_registration_attribution_source_idx
    on event_registration_attribution (event_id, source);
create index if not exists event_registration_attribution_referral_code_idx
    on event_registration_attribution (event_id, referral_code);

create table if not exists event_finance_entry (
    event_finance_entry_id uuid primary key default gen_random_uuid(),
    event_id uuid not null references event (event_id) on delete cascade,
    created_by uuid references "user" (user_id) on delete set null,
    kind text not null check (kind in ('income', 'expense')),
    category text not null check (btrim(category) <> ''),
    description text,
    amount_minor bigint not null check (amount_minor > 0),
    currency_code text not null check (currency_code ~ '^[A-Z]{3}$'),
    occurred_at date not null default current_date,
    created_at timestamptz not null default current_timestamp,
    check (description is null or btrim(description) <> '')
);

create index if not exists event_finance_entry_event_occurred_idx
    on event_finance_entry (event_id, occurred_at desc, created_at desc);
