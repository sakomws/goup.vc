-- Mock interview practice matching for GOUP members.

create table if not exists mock_interview_profile (
    user_id uuid primary key references "user" (user_id) on delete cascade,
    role_intent text not null check (role_intent in ('interviewee', 'interviewer', 'both')),
    timezone_region text not null check (
        timezone_region in ('aze', 'eu', 'usa_canada', 'asia', 'other')
    ),
    seniority text not null check (
        seniority in ('junior', 'mid', 'senior', 'staff_plus')
    ),
    interview_types text[] default '{}'::text[] not null,
    target_company_types text[] default '{}'::text[] not null,
    availability_slots jsonb default '[]'::jsonb not null,
    linkedin_url text,
    github_url text,
    resume_url text,
    enabled boolean default true not null,
    reputation_score numeric(4, 2) default 0 not null,
    completed_sessions integer default 0 not null,
    interviewer_badge boolean default false not null,
    created_at timestamp with time zone default current_timestamp not null,
    updated_at timestamp with time zone
);

create index if not exists mock_interview_profile_enabled_role_intent_idx
on mock_interview_profile (enabled, role_intent)
where enabled = true;

create index if not exists mock_interview_profile_interview_types_idx
on mock_interview_profile using gin (interview_types);

create index if not exists mock_interview_profile_target_company_types_idx
on mock_interview_profile using gin (target_company_types);

create table if not exists mock_interview_request (
    mock_interview_request_id uuid default gen_random_uuid() primary key,
    interviewee_user_id uuid not null references "user" (user_id) on delete cascade,
    interviewer_user_id uuid not null references "user" (user_id) on delete cascade,
    interview_type text not null check (btrim(interview_type) <> ''),
    message text,
    status text not null default 'pending' check (
        status in ('pending', 'accepted', 'declined', 'cancelled', 'completed', 'no_show')
    ),
    created_at timestamp with time zone default current_timestamp not null,
    responded_at timestamp with time zone,
    check (interviewee_user_id <> interviewer_user_id)
);

create index if not exists mock_interview_request_interviewee_created_at_idx
on mock_interview_request (interviewee_user_id, created_at desc);

create index if not exists mock_interview_request_interviewer_status_created_at_idx
on mock_interview_request (interviewer_user_id, status, created_at desc);

create table if not exists mock_interview_session (
    mock_interview_session_id uuid default gen_random_uuid() primary key,
    mock_interview_request_id uuid not null unique references mock_interview_request (
        mock_interview_request_id
    ) on delete cascade,
    meeting_url text,
    scheduled_at timestamp with time zone,
    status text not null default 'scheduled' check (
        status in ('scheduled', 'completed', 'cancelled')
    ),
    created_at timestamp with time zone default current_timestamp not null,
    completed_at timestamp with time zone
);

create table if not exists mock_interview_feedback (
    mock_interview_feedback_id uuid default gen_random_uuid() primary key,
    mock_interview_session_id uuid not null references mock_interview_session (
        mock_interview_session_id
    ) on delete cascade,
    reviewer_user_id uuid not null references "user" (user_id) on delete cascade,
    reviewee_user_id uuid not null references "user" (user_id) on delete cascade,
    reviewer_role text not null check (reviewer_role in ('interviewer', 'interviewee')),
    communication smallint check (communication between 1 and 5),
    technical_depth smallint check (technical_depth between 1 and 5),
    problem_solving smallint check (problem_solving between 1 and 5),
    role_readiness smallint check (role_readiness between 1 and 5),
    helpfulness smallint check (helpfulness between 1 and 5),
    feedback_quality smallint check (feedback_quality between 1 and 5),
    would_recommend boolean,
    suggested_next_steps text,
    created_at timestamp with time zone default current_timestamp not null,
    unique (mock_interview_session_id, reviewer_user_id)
);

create index if not exists mock_interview_feedback_reviewee_created_at_idx
on mock_interview_feedback (reviewee_user_id, created_at desc);
