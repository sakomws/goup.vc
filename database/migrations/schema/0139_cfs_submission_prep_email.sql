-- Records each speaker prep email sent for an approved submission.

create table cfs_submission_prep_email (
    cfs_submission_id uuid not null references cfs_submission on delete cascade,
    prep_step text not null,
    created_at timestamptz not null default current_timestamp,
    sent_by uuid not null references "user",

    primary key (cfs_submission_id, prep_step),
    constraint cfs_submission_prep_email_step_check check (
        prep_step in ('listed', 'profile', 'slides', 'schedule')
    )
);
