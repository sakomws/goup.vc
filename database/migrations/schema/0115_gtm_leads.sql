-- GTM pipeline for alliance and group organizers: leads, activity, and
-- human-reviewed agent drafts. Agents never advance a stage on their own.

create table if not exists gtm_lead (
    gtm_lead_id uuid default gen_random_uuid() not null,
    alliance_id uuid not null references alliance (alliance_id) on delete cascade,
    group_id uuid references "group" (group_id) on delete set null,
    kind text not null,
    stage text not null default 'lead_generation',
    name text not null,
    org_name text,
    email text,
    website_url text,
    linkedin_url text,
    landscape_entry_id uuid references landscape_entry (landscape_entry_id) on delete set null,
    user_id uuid references "user" (user_id) on delete set null,
    group_sponsor_id uuid references group_sponsor (group_sponsor_id) on delete set null,
    owner_user_id uuid references "user" (user_id) on delete set null,
    score integer,
    estimated_value_cents bigint,
    currency text,
    next_action_at timestamp with time zone,
    renewal_at timestamp with time zone,
    lost_reason text,
    source text not null default 'manual',
    notes text,
    payload jsonb not null default '{}'::jsonb,
    created_at timestamp with time zone default current_timestamp not null,
    updated_at timestamp with time zone default current_timestamp not null,
    constraint gtm_lead_pkey primary key (gtm_lead_id),
    constraint gtm_lead_kind_check check (
        kind in ('sponsor', 'startup', 'investor', 'speaker', 'organizer')
    ),
    constraint gtm_lead_stage_check check (
        stage in (
            'lead_generation',
            'reachout',
            'get_response',
            'qualification',
            'proposal',
            'negotiation',
            'won',
            'lost',
            'delivered',
            'renewal'
        )
    ),
    constraint gtm_lead_source_check check (
        source in ('manual', 'landscape', 'member', 'discovery')
    ),
    constraint gtm_lead_name_check check (btrim(name) <> ''),
    constraint gtm_lead_email_check check (email is null or btrim(email) <> ''),
    constraint gtm_lead_website_url_check check (website_url is null or btrim(website_url) <> ''),
    constraint gtm_lead_score_check check (score is null or (score >= 0 and score <= 100))
);

create index if not exists gtm_lead_alliance_stage_idx
    on gtm_lead (alliance_id, stage, updated_at desc);

create index if not exists gtm_lead_group_stage_idx
    on gtm_lead (group_id, stage, updated_at desc)
    where group_id is not null;

create unique index if not exists gtm_lead_alliance_email_key
    on gtm_lead (alliance_id, lower(email))
    where email is not null and btrim(email) <> '';

create unique index if not exists gtm_lead_alliance_website_key
    on gtm_lead (alliance_id, lower(website_url))
    where website_url is not null and btrim(website_url) <> '';

create table if not exists gtm_lead_activity (
    gtm_lead_activity_id uuid default gen_random_uuid() not null,
    gtm_lead_id uuid references gtm_lead (gtm_lead_id) on delete cascade,
    actor_user_id uuid references "user" (user_id) on delete set null,
    agent_id text,
    kind text not null,
    body text,
    details jsonb not null default '{}'::jsonb,
    created_at timestamp with time zone default current_timestamp not null,
    constraint gtm_lead_activity_pkey primary key (gtm_lead_activity_id),
    constraint gtm_lead_activity_kind_check check (
        kind in (
            'stage_change',
            'note',
            'outreach',
            'response',
            'draft_created',
            'draft_approved',
            'draft_rejected',
            'side_effect'
        )
    )
);

create index if not exists gtm_lead_activity_lead_created_idx
    on gtm_lead_activity (gtm_lead_id, created_at desc);

create table if not exists gtm_agent_draft (
    gtm_agent_draft_id uuid default gen_random_uuid() not null,
    alliance_id uuid not null references alliance (alliance_id) on delete cascade,
    group_id uuid references "group" (group_id) on delete set null,
    gtm_lead_id uuid references gtm_lead (gtm_lead_id) on delete cascade,
    agent_id text not null,
    status text not null default 'pending',
    title text not null,
    body text not null default '',
    suggested_stage text,
    payload jsonb not null default '{}'::jsonb,
    created_at timestamp with time zone default current_timestamp not null,
    reviewed_at timestamp with time zone,
    reviewed_by uuid references "user" (user_id) on delete set null,
    constraint gtm_agent_draft_pkey primary key (gtm_agent_draft_id),
    constraint gtm_agent_draft_agent_id_check check (
        agent_id in (
            'lead_generation',
            'reachout',
            'get_response',
            'qualification',
            'proposal',
            'negotiation',
            'won_lost',
            'delivered',
            'renewal'
        )
    ),
    constraint gtm_agent_draft_status_check check (
        status in ('pending', 'approved', 'rejected', 'superseded')
    ),
    constraint gtm_agent_draft_title_check check (btrim(title) <> ''),
    constraint gtm_agent_draft_suggested_stage_check check (
        suggested_stage is null or suggested_stage in (
            'lead_generation',
            'reachout',
            'get_response',
            'qualification',
            'proposal',
            'negotiation',
            'won',
            'lost',
            'delivered',
            'renewal'
        )
    )
);

create index if not exists gtm_agent_draft_alliance_status_idx
    on gtm_agent_draft (alliance_id, status, created_at desc);

create index if not exists gtm_agent_draft_lead_status_idx
    on gtm_agent_draft (gtm_lead_id, status, created_at desc)
    where gtm_lead_id is not null;

create unique index if not exists gtm_agent_draft_pending_lead_agent_key
    on gtm_agent_draft (gtm_lead_id, agent_id)
    where status = 'pending' and gtm_lead_id is not null;

insert into alliance_permission (alliance_permission_id, display_name)
values ('alliance.gtm.write', 'GTM Write')
on conflict (alliance_permission_id) do nothing;

insert into group_permission (group_permission_id, display_name)
values ('group.gtm.write', 'GTM Write')
on conflict (group_permission_id) do nothing;

insert into alliance_role_alliance_permission (alliance_permission_id, alliance_role_id)
values
    ('alliance.gtm.write', 'admin'),
    ('alliance.gtm.write', 'groups-manager')
on conflict do nothing;

insert into alliance_role_group_permission (alliance_role_id, group_permission_id)
values
    ('admin', 'group.gtm.write'),
    ('groups-manager', 'group.gtm.write')
on conflict do nothing;

insert into group_role_group_permission (group_permission_id, group_role_id)
values ('group.gtm.write', 'admin')
on conflict do nothing;
