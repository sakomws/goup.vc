-- Group-owned collaboration workspaces. This migration intentionally depends
-- only on schema available before 0123 so it can ship independently.

create table collaboration_project (
    collaboration_project_id uuid primary key default gen_random_uuid(),
    group_id uuid not null references "group" (group_id) on delete cascade,
    created_by uuid not null references "user" (user_id),
    landscape_entry_id uuid references landscape_entry (landscape_entry_id) on delete set null,
    group_accelerator_cohort_id uuid references group_accelerator_cohort (group_accelerator_cohort_id) on delete set null,
    slug text not null,
    name text not null,
    summary text not null,
    description text,
    lifecycle text default 'proposed' not null
        check (lifecycle in ('proposed', 'active', 'paused', 'completed', 'archived')),
    visibility text default 'members' not null
        check (visibility in ('private', 'members', 'public')),
    website_url text,
    repository_url text,
    cover_image_url text,
    starts_on date,
    target_ends_on date,
    completed_at timestamptz,
    created_at timestamptz default current_timestamp not null,
    updated_at timestamptz default current_timestamp not null,
    constraint collaboration_project_slug_check check (slug ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$'),
    constraint collaboration_project_name_check check (btrim(name) <> ''),
    constraint collaboration_project_summary_check check (btrim(summary) <> ''),
    constraint collaboration_project_dates_check check (
        starts_on is null or target_ends_on is null or starts_on <= target_ends_on
    ),
    constraint collaboration_project_completion_check check (
        (lifecycle <> 'completed' or completed_at is not null)
        and (completed_at is null or lifecycle in ('completed', 'archived'))
    ),
    unique (group_id, slug)
);

create index collaboration_project_group_lifecycle_idx
on collaboration_project (group_id, lifecycle, updated_at desc);

create table collaboration_project_member (
    collaboration_project_member_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    user_id uuid not null references "user" (user_id) on delete cascade,
    role text not null check (role in ('owner', 'contributor', 'viewer')),
    invitation_status text default 'pending' not null
        check (invitation_status in ('pending', 'accepted', 'declined', 'revoked')),
    invited_by uuid references "user" (user_id) on delete set null,
    invited_at timestamptz default current_timestamp not null,
    responded_at timestamptz,
    invitation_attempt integer default 1 not null check (invitation_attempt > 0),
    unique (collaboration_project_id, user_id)
);

create unique index collaboration_project_one_owner_idx
on collaboration_project_member (collaboration_project_id)
where role = 'owner' and invitation_status = 'accepted';

create table collaboration_project_goal (
    collaboration_project_goal_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    created_by uuid not null references "user" (user_id),
    title text not null check (btrim(title) <> ''),
    description text,
    status text default 'open' not null check (status in ('open', 'achieved', 'cancelled')),
    target_value numeric,
    current_value numeric,
    unit text,
    due_on date,
    created_at timestamptz default current_timestamp not null,
    updated_at timestamptz default current_timestamp not null
);

create table collaboration_project_task (
    collaboration_project_task_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    collaboration_project_goal_id uuid references collaboration_project_goal (collaboration_project_goal_id) on delete set null,
    assigned_member_id uuid references collaboration_project_member (collaboration_project_member_id) on delete set null,
    created_by uuid not null references "user" (user_id),
    title text not null check (btrim(title) <> ''),
    description text,
    status text default 'todo' not null check (status in ('todo', 'in_progress', 'blocked', 'done', 'cancelled')),
    due_on date,
    completed_at timestamptz,
    created_at timestamptz default current_timestamp not null,
    updated_at timestamptz default current_timestamp not null,
    constraint collaboration_project_task_completion_check check (
        (status = 'done') = (completed_at is not null)
    )
);

create index collaboration_project_task_project_status_idx
on collaboration_project_task (collaboration_project_id, status, due_on);

create table collaboration_project_update (
    collaboration_project_update_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    collaboration_project_member_id uuid not null references collaboration_project_member (collaboration_project_member_id) on delete cascade,
    body text not null check (btrim(body) <> ''),
    blockers text,
    next_steps text,
    created_at timestamptz default current_timestamp not null
);

create index collaboration_project_update_chronological_idx
on collaboration_project_update (collaboration_project_id, created_at, collaboration_project_update_id);

create table collaboration_project_activity (
    collaboration_project_activity_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    actor_user_id uuid references "user" (user_id) on delete set null,
    kind text not null check (btrim(kind) <> ''),
    subject_type text not null check (btrim(subject_type) <> ''),
    subject_id uuid,
    data jsonb default '{}'::jsonb not null,
    created_at timestamptz default current_timestamp not null
);

create index collaboration_project_activity_timeline_idx
on collaboration_project_activity (collaboration_project_id, created_at desc, collaboration_project_activity_id desc);

create or replace function prevent_collaboration_activity_mutation()
returns trigger as $$
begin
    raise exception 'collaboration project activity is append-only';
end;
$$ language plpgsql;

create trigger collaboration_project_activity_append_only
before update or delete on collaboration_project_activity
for each row execute function prevent_collaboration_activity_mutation();

create or replace function validate_collaboration_project_task_references()
returns trigger as $$
begin
    if new.collaboration_project_goal_id is not null and not exists (
        select 1
        from collaboration_project_goal g
        where g.collaboration_project_goal_id = new.collaboration_project_goal_id
          and g.collaboration_project_id = new.collaboration_project_id
    ) then
        raise exception 'task goal must belong to the same project';
    end if;
    if new.assigned_member_id is not null and not exists (
        select 1
        from collaboration_project_member m
        where m.collaboration_project_member_id = new.assigned_member_id
          and m.collaboration_project_id = new.collaboration_project_id
          and m.invitation_status = 'accepted'
    ) then
        raise exception 'task assignee must be an accepted member of the same project';
    end if;
    return new;
end;
$$ language plpgsql;

create or replace function validate_collaboration_project_update_reference()
returns trigger as $$
begin
    if not exists (
        select 1
        from collaboration_project_member m
        where m.collaboration_project_member_id = new.collaboration_project_member_id
          and m.collaboration_project_id = new.collaboration_project_id
          and m.invitation_status = 'accepted'
          and m.role in ('owner', 'contributor')
    ) then
        raise exception 'update author must be an accepted owner or contributor of the same project';
    end if;
    return new;
end;
$$ language plpgsql;

create or replace function validate_collaboration_office_hour_booking_reference()
returns trigger as $$
begin
    if not exists (
        select 1
        from collaboration_office_hour_session s
        join collaboration_project_member m
          on m.collaboration_project_member_id = new.collaboration_project_member_id
         and m.collaboration_project_id = s.collaboration_project_id
         and m.invitation_status = 'accepted'
        where s.collaboration_office_hour_session_id = new.collaboration_office_hour_session_id
    ) then
        raise exception 'booking member must be accepted in the session project';
    end if;
    return new;
end;
$$ language plpgsql;

create trigger collaboration_project_task_reference_check
before insert or update of collaboration_project_id, collaboration_project_goal_id, assigned_member_id
on collaboration_project_task
for each row execute function validate_collaboration_project_task_references();

create trigger collaboration_project_update_reference_check
before insert or update of collaboration_project_id, collaboration_project_member_id
on collaboration_project_update
for each row execute function validate_collaboration_project_update_reference();

create table collaboration_project_outcome (
    collaboration_project_outcome_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    recorded_by uuid not null references "user" (user_id),
    title text not null check (btrim(title) <> ''),
    narrative text,
    metric_name text,
    baseline_value numeric,
    target_value numeric,
    achieved_value numeric,
    unit text,
    measured_on date,
    created_at timestamptz default current_timestamp not null
);

create table collaboration_project_evidence (
    collaboration_project_evidence_id uuid primary key default gen_random_uuid(),
    collaboration_project_outcome_id uuid not null references collaboration_project_outcome (collaboration_project_outcome_id) on delete cascade,
    added_by uuid not null references "user" (user_id),
    label text not null check (btrim(label) <> ''),
    url text not null check (btrim(url) <> ''),
    created_at timestamptz default current_timestamp not null
);

create table collaboration_office_hour_session (
    collaboration_office_hour_session_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    expert_user_id uuid not null references "user" (user_id),
    created_by uuid not null references "user" (user_id),
    title text not null check (btrim(title) <> ''),
    description text,
    meeting_url text,
    starts_at timestamptz not null,
    ends_at timestamptz not null,
    capacity integer default 1 not null check (capacity > 0),
    status text default 'scheduled' not null check (status in ('scheduled', 'cancelled', 'completed')),
    created_at timestamptz default current_timestamp not null,
    constraint collaboration_office_hour_dates_check check (starts_at < ends_at)
);

create table collaboration_office_hour_booking (
    collaboration_office_hour_booking_id uuid primary key default gen_random_uuid(),
    collaboration_office_hour_session_id uuid not null references collaboration_office_hour_session (collaboration_office_hour_session_id) on delete cascade,
    collaboration_project_member_id uuid not null references collaboration_project_member (collaboration_project_member_id) on delete cascade,
    question text,
    status text default 'booked' not null check (status in ('booked', 'cancelled', 'attended', 'no_show')),
    booked_at timestamptz default current_timestamp not null,
    cancelled_at timestamptz,
    booking_attempt integer default 1 not null check (booking_attempt > 0),
    unique (collaboration_office_hour_session_id, collaboration_project_member_id)
);

create trigger collaboration_office_hour_booking_reference_check
before insert or update of collaboration_office_hour_session_id, collaboration_project_member_id
on collaboration_office_hour_booking
for each row execute function validate_collaboration_office_hour_booking_reference();

-- Delivery keys make invitation, booking, and reminder scheduling retry-safe.
create table collaboration_notification_delivery (
    collaboration_notification_delivery_id uuid primary key default gen_random_uuid(),
    collaboration_project_id uuid not null references collaboration_project (collaboration_project_id) on delete cascade,
    recipient_user_id uuid not null references "user" (user_id) on delete cascade,
    kind text not null check (kind in ('invitation', 'booking', 'session_reminder')),
    idempotency_key text not null check (btrim(idempotency_key) <> ''),
    payload jsonb default '{}'::jsonb not null,
    created_at timestamptz default current_timestamp not null,
    unique (kind, idempotency_key, recipient_user_id)
);

insert into group_permission (group_permission_id, display_name)
values ('group.projects.write', 'Projects Write')
on conflict (group_permission_id) do nothing;

insert into alliance_role_group_permission (alliance_role_id, group_permission_id)
values
    ('admin', 'group.projects.write'),
    ('groups-manager', 'group.projects.write')
on conflict do nothing;

insert into group_role_group_permission (group_permission_id, group_role_id)
values ('group.projects.write', 'admin')
on conflict do nothing;
