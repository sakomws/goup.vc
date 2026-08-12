-- Allow an event to override its group or alliance image in social link previews.
alter table event
    add column if not exists og_image_url text check (btrim(og_image_url) <> '');

create index if not exists event_og_image_url_idx on event (og_image_url)
where og_image_url is not null;
