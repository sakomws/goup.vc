-- Converts an input hostname to the canonical value stored in custom_domain.
create or replace function normalize_custom_domain_hostname(
    p_hostname text
)
returns text
immutable
strict
language sql
as $$
    select lower(rtrim(btrim(p_hostname), '.'));
$$;
