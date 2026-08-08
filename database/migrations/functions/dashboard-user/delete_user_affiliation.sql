-- delete_user_affiliation removes one of the user's affiliations.
create or replace function delete_user_affiliation(
    p_user_id uuid,
    p_user_affiliation_id uuid
)
returns void as $$
begin
    delete from user_affiliation
    where user_affiliation_id = p_user_affiliation_id
      and user_id = p_user_id;

    if not found then
        raise exception 'affiliation not found'
            using errcode = 'no_data_found';
    end if;

    perform insert_audit_log(
        'user_affiliation_deleted',
        p_user_id,
        'user_affiliation',
        p_user_affiliation_id
    );
end;
$$ language plpgsql;
