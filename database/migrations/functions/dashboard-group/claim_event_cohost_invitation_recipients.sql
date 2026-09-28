-- Atomically claims verified, opted-in organizers who can decide a pending
-- co-host invitation. The delivery ledger suppresses retries and duplicate
-- group/alliance team membership.
create or replace function claim_event_cohost_invitation_recipients(
    p_event_cohost_id uuid
)
returns uuid[] as $$
    with invitation as (
        select ec.event_id, ec.cohost_group_id, g.alliance_id
        from event_cohost ec
        join "group" g on g.group_id = ec.cohost_group_id
        where ec.event_cohost_id = p_event_cohost_id
          and ec.status = 'pending'
          and g.active = true
          and g.deleted = false
    ),
    candidate as (
        select gt.user_id
        from invitation i
        join group_team gt on gt.group_id = i.cohost_group_id
        where gt.accepted = true

        union

        select ct.user_id
        from invitation i
        join alliance_team ct on ct.alliance_id = i.alliance_id
        where ct.accepted = true
    ),
    eligible as (
        select distinct i.event_id, i.cohost_group_id, c.user_id
        from invitation i
        join candidate c on true
        join "user" u on u.user_id = c.user_id
        where u.registration_status = 'registered'
          and u.email_verified = true
          and coalesce(u.optional_notifications_enabled, true) = true
          and user_has_group_permission(
              i.alliance_id,
              i.cohost_group_id,
              c.user_id,
              'group.events.write'
          )
    ),
    claimed as (
        insert into event_cohost_delivery (
            event_id,
            cohost_group_id,
            user_id,
            delivery_kind
        )
        select event_id, cohost_group_id, user_id, 'cohost-invitation'
        from eligible
        on conflict do nothing
        returning user_id
    )
    select coalesce(array_agg(user_id order by user_id), '{}')
    from claimed;
$$ language sql;
