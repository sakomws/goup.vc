-- get_public_user_provider removes private external identity metadata.
create or replace function get_public_user_provider(
    p_provider jsonb
) returns jsonb as $$
    select case
        when jsonb_typeof(p_provider -> 'github') = 'object'
            then jsonb_build_object('github', p_provider -> 'github')
        else null
    end;
$$ language sql immutable;
