-- Activates a verified domain after its certificate/routing setup is ready.
create or replace function mark_custom_domain_active(
    p_custom_domain_id uuid
)
returns boolean
language plpgsql
as $$
begin
    update custom_domain
    set activated_at = coalesce(activated_at, current_timestamp),
        updated_at = current_timestamp
    where custom_domain_id = p_custom_domain_id
      and verified_at is not null;

    return found;
end;
$$;
