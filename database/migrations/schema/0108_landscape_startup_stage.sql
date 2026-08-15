-- Structured funding stage for startup entries in the public landscape.
alter table landscape_entry
    add column if not exists stage text check (
        stage is null
        or stage in ('pre_seed', 'seed', 'series_a', 'series_b', 'series_c_plus', 'growth', 'public')
    );

create index if not exists landscape_entry_startup_stage_idx
    on landscape_entry (stage)
    where kind = 'startup' and stage is not null;
