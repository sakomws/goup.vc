-- Inserts a baseline event category row with placeholder plumbing.
create or replace function fx_event_category(
    p_event_category_id uuid,
    p_alliance_id uuid,
    p_overrides jsonb default '{}'::jsonb
)
returns void as $$
begin
    perform fx_insert_row('event_category', jsonb_build_object(
        'alliance_id', p_alliance_id,
        'event_category_id', p_event_category_id,
        'name', 'Fixture Event Category ' || p_event_category_id
    ) || p_overrides);
end;
$$ language plpgsql;
