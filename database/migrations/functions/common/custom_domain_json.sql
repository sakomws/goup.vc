-- Produces the dashboard representation of a custom domain.
create or replace function custom_domain_json(
    p_custom_domain custom_domain
)
returns jsonb
immutable
strict
language sql
as $$
    select jsonb_strip_nulls(jsonb_build_object(
        'activated_at', p_custom_domain.activated_at,
        'custom_domain_id', p_custom_domain.custom_domain_id,
        'event_id', p_custom_domain.event_id,
        'group_id', p_custom_domain.group_id,
        'hostname', p_custom_domain.hostname,
        'verified_at', p_custom_domain.verified_at,
        'verification_token', p_custom_domain.verification_token,
        'verification_token_created_at', p_custom_domain.verification_token_created_at
    ));
$$;
