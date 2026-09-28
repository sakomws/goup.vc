-- Authenticated, event-scoped post-event surveys.

create table event_survey (
    event_id uuid not null references event (event_id) on delete cascade,
    audience text not null check (audience in ('attendee', 'speaker', 'sponsor-contact')),
    questions jsonb not null,
    enabled boolean not null default true,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp,
    primary key (event_id, audience)
);

create table event_survey_response (
    event_id uuid not null,
    audience text not null,
    user_id uuid not null references "user" (user_id) on delete cascade,
    answers jsonb not null,
    submitted_at timestamptz not null default current_timestamp,
    primary key (event_id, audience, user_id),
    foreign key (event_id, audience)
        references event_survey (event_id, audience) on delete cascade
);

create index event_survey_response_dashboard_idx
    on event_survey_response (event_id, audience, submitted_at desc);

-- This is both an idempotency key and durable enqueue state. Delivery state remains
-- in the shared notification queue.
create table event_survey_notification (
    event_id uuid not null,
    audience text not null,
    user_id uuid not null references "user" (user_id) on delete cascade,
    kind text not null check (kind in ('request', 'reminder')),
    enqueued_at timestamptz not null default current_timestamp,
    primary key (event_id, audience, user_id, kind),
    foreign key (event_id, audience)
        references event_survey (event_id, audience) on delete cascade
);

insert into notification_kind (notification_kind_id, name, optional_notification)
values
    ('2d7fd58d-7d03-4ac2-a25e-a87565d762be', 'event-survey-request', true),
    ('4869152b-1d0a-44e2-b6db-3936c561cb70', 'event-survey-reminder', true)
on conflict (name) do nothing;
