-- Completes an organizer-confirmed external (off-Stripe) ticket purchase.
create or replace function complete_external_event_purchase(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_user_id uuid,
    p_details text default null
)
returns jsonb as $$
declare
    v_alliance_id uuid;
    v_event_canceled boolean;
    v_event_deleted boolean;
    v_event_ends_at timestamptz;
    v_event_published boolean;
    v_event_starts_at timestamptz;
    v_group_active boolean;
    v_purchase_id uuid;
    v_status text;
begin
    -- Lock the event and group before completing the external purchase
    select
        e.canceled,
        e.deleted,
        e.ends_at,
        g.alliance_id,
        e.published,
        e.starts_at,
        g.active
    into
        v_event_canceled,
        v_event_deleted,
        v_event_ends_at,
        v_alliance_id,
        v_event_published,
        v_event_starts_at,
        v_group_active
    from event e
    join "group" g on g.group_id = e.group_id
    where e.event_id = p_event_id
    and e.group_id = p_group_id
    for update of e, g;

    if not found then
        raise exception 'event not found';
    end if;

    if not v_group_active
       or v_event_deleted
       or not v_event_published
       or v_event_canceled
       or (
           coalesce(v_event_ends_at, v_event_starts_at) is not null
           and coalesce(v_event_ends_at, v_event_starts_at) <= current_timestamp
       ) then
        raise exception 'event not found or inactive';
    end if;

    -- Lock the latest pending external purchase for this attendee
    select
        ep.event_purchase_id,
        ep.status
    into
        v_purchase_id,
        v_status
    from event_purchase ep
    where ep.event_id = p_event_id
    and ep.user_id = p_user_id
    and ep.charge_model = 'external'
    and ep.status = 'pending'
    order by ep.created_at desc, ep.event_purchase_id desc
    limit 1
    for update of ep;

    if not found then
        raise exception 'pending external purchase not found';
    end if;

    if v_status <> 'pending' then
        raise exception 'purchase is no longer pending';
    end if;

    insert into event_attendee (event_id, user_id)
    values (p_event_id, p_user_id)
    on conflict (event_id, user_id) do update
    set status = 'confirmed'
    where event_attendee.status in (
        'confirmed',
        'invitation-canceled',
        'registration-questions-pending'
    );

    if not found then
        raise exception 'attendee cannot be confirmed for this event';
    end if;

    update event_purchase
    set
        completed_at = current_timestamp,
        external_payment_details = nullif(btrim(p_details), ''),
        external_payment_marked_by_user_id = p_actor_user_id,
        hold_expires_at = null,
        status = 'completed',
        updated_at = current_timestamp
    where event_purchase_id = v_purchase_id;

    perform insert_audit_log(
        'event_external_payment_confirmed',
        p_actor_user_id,
        'event',
        p_event_id,
        v_alliance_id,
        p_group_id,
        p_event_id,
        jsonb_build_object(
            'event_purchase_id', v_purchase_id,
            'user_id', p_user_id
        )
    );

    return jsonb_build_object(
        'alliance_id', v_alliance_id,
        'event_id', p_event_id,
        'event_purchase_id', v_purchase_id,
        'user_id', p_user_id
    );
end;
$$ language plpgsql;
