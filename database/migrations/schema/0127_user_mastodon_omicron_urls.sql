alter table "user"
add column if not exists mastodon_url text,
add column if not exists omicron_url text;

do $$
begin
    if not exists (
        select 1
        from pg_constraint
        where conname = 'user_mastodon_url_check'
    ) then
        alter table "user"
        add constraint user_mastodon_url_check
        check (btrim(mastodon_url) <> '');
    end if;

    if not exists (
        select 1
        from pg_constraint
        where conname = 'user_omicron_url_check'
    ) then
        alter table "user"
        add constraint user_omicron_url_check
        check (btrim(omicron_url) <> '');
    end if;
end $$;
