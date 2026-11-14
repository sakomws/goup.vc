-- Inserts a baseline alliance row with placeholder plumbing.
create or replace function fx_alliance(
    p_alliance_id uuid,
    p_overrides jsonb default '{}'::jsonb
)
returns void as $$
begin
    perform fx_insert_row('alliance', jsonb_build_object(
        'banner_mobile_url', 'https://fixture.test/alliance-banner-mobile.png',
        'banner_url', 'https://fixture.test/alliance-banner.png',
        'alliance_id', p_alliance_id,
        'description', 'Fixture alliance',
        'display_name', 'Fixture Alliance ' || p_alliance_id,
        'logo_url', 'https://fixture.test/alliance-logo.png',
        'name', 'fixture-alliance-' || p_alliance_id
    ) || p_overrides);
end;
$$ language plpgsql;
