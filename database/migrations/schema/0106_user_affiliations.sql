create table if not exists user_affiliation (
    user_affiliation_id uuid default gen_random_uuid() primary key,
    user_id uuid not null references "user" (user_id) on delete cascade,
    landscape_entry_id uuid not null references landscape_entry (landscape_entry_id) on delete cascade,
    role text not null check (
        role in (
            'founder',
            'co_founder',
            'executive',
            'maintainer',
            'contributor',
            'representative',
            'other'
        )
    ),
    created_at timestamp with time zone default current_timestamp not null,
    unique (user_id, landscape_entry_id)
);

create index if not exists user_affiliation_landscape_entry_id_idx
on user_affiliation (landscape_entry_id);
