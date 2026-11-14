-- Compatibility alias for upstream tests while GOUP uses alliance terminology.
create or replace function fx_community(
    p_community_id uuid,
    p_overrides jsonb default '{}'::jsonb
)
returns void as $$
begin
    perform fx_alliance(p_community_id, p_overrides);
end;
$$ language plpgsql;
