-- Sponsor campaign operations layered onto the human-approved GTM pipeline.

alter table gtm_lead_activity
    drop constraint if exists gtm_lead_activity_kind_check;
alter table gtm_lead_activity
    add constraint gtm_lead_activity_kind_check check (
        kind in (
            'stage_change', 'note', 'call', 'email', 'meeting', 'outreach', 'response',
            'draft_created', 'draft_approved', 'draft_rejected', 'side_effect'
        )
    );

create table if not exists gtm_sponsor_package (
    gtm_sponsor_package_id uuid default gen_random_uuid() primary key,
    alliance_id uuid not null references alliance (alliance_id) on delete cascade,
    group_id uuid references "group" (group_id) on delete cascade,
    name text not null check (btrim(name) <> ''),
    description text,
    price_cents bigint not null check (price_cents >= 0),
    currency text not null check (currency ~ '^[A-Z]{3}$'),
    billing_period text not null default 'one_time'
        check (billing_period in ('one_time', 'monthly', 'quarterly', 'annual')),
    active boolean not null default true,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp
);

create unique index if not exists gtm_sponsor_package_scope_name_key
    on gtm_sponsor_package (alliance_id, coalesce(group_id, '00000000-0000-0000-0000-000000000000'::uuid), lower(name));

create table if not exists gtm_sponsor_package_deliverable (
    gtm_sponsor_package_deliverable_id uuid default gen_random_uuid() primary key,
    gtm_sponsor_package_id uuid not null references gtm_sponsor_package (gtm_sponsor_package_id) on delete cascade,
    name text not null check (btrim(name) <> ''),
    description text,
    quantity integer not null default 1 check (quantity > 0),
    sort_order integer not null default 0
);

create table if not exists gtm_sponsor_contact (
    gtm_sponsor_contact_id uuid default gen_random_uuid() primary key,
    gtm_lead_id uuid not null references gtm_lead (gtm_lead_id) on delete cascade,
    user_id uuid references "user" (user_id) on delete set null,
    name text not null check (btrim(name) <> ''),
    email text,
    title text,
    is_primary boolean not null default false,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp,
    constraint gtm_sponsor_contact_email_check check (email is null or btrim(email) <> '')
);

create unique index if not exists gtm_sponsor_contact_lead_email_key
    on gtm_sponsor_contact (gtm_lead_id, lower(email))
    where email is not null;
create unique index if not exists gtm_sponsor_contact_primary_key
    on gtm_sponsor_contact (gtm_lead_id)
    where is_primary;

create table if not exists gtm_sponsor_proposal (
    gtm_sponsor_proposal_id uuid default gen_random_uuid() primary key,
    gtm_lead_id uuid not null references gtm_lead (gtm_lead_id) on delete cascade,
    gtm_sponsor_package_id uuid references gtm_sponsor_package (gtm_sponsor_package_id) on delete set null,
    created_by uuid references "user" (user_id) on delete set null,
    status text not null default 'draft'
        check (status in ('draft', 'sent', 'accepted', 'declined', 'expired', 'withdrawn')),
    title text not null check (btrim(title) <> ''),
    package_snapshot jsonb not null,
    amount_cents bigint not null check (amount_cents >= 0),
    currency text not null check (currency ~ '^[A-Z]{3}$'),
    valid_until date,
    sent_at timestamptz,
    accepted_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp
);

create index if not exists gtm_sponsor_proposal_lead_created_idx
    on gtm_sponsor_proposal (gtm_lead_id, created_at desc);

create table if not exists gtm_task (
    gtm_task_id uuid default gen_random_uuid() primary key,
    gtm_lead_id uuid not null references gtm_lead (gtm_lead_id) on delete cascade,
    assigned_user_id uuid references "user" (user_id) on delete set null,
    created_by uuid references "user" (user_id) on delete set null,
    title text not null check (btrim(title) <> ''),
    notes text,
    due_at timestamptz not null,
    state text not null default 'open' check (state in ('open', 'completed', 'cancelled')),
    reminder_key text,
    completed_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp
);

create unique index if not exists gtm_task_reminder_key_key
    on gtm_task (gtm_lead_id, reminder_key) where reminder_key is not null;
create index if not exists gtm_task_due_open_idx
    on gtm_task (due_at, gtm_task_id) where state = 'open';

create table if not exists gtm_sponsor_deliverable (
    gtm_sponsor_deliverable_id uuid default gen_random_uuid() primary key,
    gtm_lead_id uuid not null references gtm_lead (gtm_lead_id) on delete cascade,
    gtm_sponsor_proposal_id uuid references gtm_sponsor_proposal (gtm_sponsor_proposal_id) on delete set null,
    title text not null check (btrim(title) <> ''),
    notes text,
    due_at timestamptz,
    state text not null default 'pending'
        check (state in ('pending', 'in_progress', 'delivered', 'waived')),
    completed_at timestamptz,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp
);

create index if not exists gtm_sponsor_deliverable_lead_state_idx
    on gtm_sponsor_deliverable (gtm_lead_id, state, due_at);

create table if not exists gtm_lost_reason (
    code text primary key,
    display_name text not null,
    sort_order integer not null,
    active boolean not null default true
);

insert into gtm_lost_reason (code, display_name, sort_order) values
    ('budget', 'Budget unavailable', 10),
    ('timing', 'Timing', 20),
    ('no_response', 'No response', 30),
    ('not_a_fit', 'Not a fit', 40),
    ('competitor', 'Selected another partner', 50),
    ('internal_change', 'Internal change', 60),
    ('other', 'Other', 100)
on conflict (code) do update
set display_name = excluded.display_name, sort_order = excluded.sort_order;

alter table gtm_lead
    add column if not exists lost_reason_detail text;

update gtm_lead
set lost_reason_detail = coalesce(lost_reason_detail, lost_reason),
    lost_reason = 'other'
where lost_reason is not null
  and lost_reason not in (
      'budget', 'timing', 'no_response', 'not_a_fit', 'competitor', 'internal_change', 'other'
  );

alter table gtm_lead
    drop constraint if exists gtm_lead_lost_reason_check;
alter table gtm_lead
    add constraint gtm_lead_lost_reason_check
    check (lost_reason is null or lost_reason in (
        'budget', 'timing', 'no_response', 'not_a_fit', 'competitor', 'internal_change', 'other'
    )) not valid;
