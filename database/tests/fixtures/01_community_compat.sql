-- Upstream-authored tests still use the former community relation name.
-- Keep that vocabulary test-only while production remains alliance-based.
create view community as
select
    alliance_id as community_id,
    name,
    display_name,
    description,
    banner_mobile_url,
    banner_url,
    logo_url,
    active,
    created_at
from alliance;
