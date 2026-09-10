-- Cancels a confirmed event attendance from the group dashboard.
create or replace function cancel_event_attendee_attendance(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_user_id uuid
) returns json as $$
declare
    v_capacity int;
    v_alliance_id uuid;
    v_is_ticketed boolean;
    v_promoted_user_ids uuid[] := array[]::uuid[];
    v_purchase_amount_minor bigint;
    v_purchase_id uuid;
begin
    -- Lock the event and verify it belongs to the selected group and can be changed
    select
        e.capacity,
        g.alliance_id,
        exists(
            select 1
            from event_ticket_type ett
            where ett.event_id = e.event_id
        )
    into
        v_capacity,
        v_alliance_id,
        v_is_ticketed
    from event e
    join "group" g using (group_id)
    where e.event_id = p_event_id
    and e.group_id = p_group_id
    and e.deleted = false
    and e.published = true
    and e.canceled = false
    and (
        coalesce(e.ends_at, e.starts_at) is null
        or coalesce(e.ends_at, e.starts_at) >= current_timestamp
    )
    for update of e;

    if not found then
        raise exception 'event not found or inactive';
    end if;

    -- Capture the latest refundable purchase so paid cancels can auto-refund
    select
        ep.amount_minor,
        ep.event_purchase_id
    into
        v_purchase_amount_minor,
        v_purchase_id
    from event_purchase ep
    where ep.event_id = p_event_id
    and ep.user_id = p_user_id
    and ep.status in ('completed', 'refund-requested')
    order by ep.created_at desc, ep.event_purchase_id desc
    limit 1;

    -- Remove only confirmed attendees
    delete from event_attendee
    where event_id = p_event_id
    and user_id = p_user_id
    and status = 'confirmed';

    if not found then
        raise exception 'confirmed event attendee not found';
    end if;

    -- Free purchases can be refunded locally; paid ones queue an organizer refund
    if v_purchase_id is not null and coalesce(v_purchase_amount_minor, 0) = 0 then
        perform refund_free_event_purchase(v_purchase_id);
        v_purchase_id := null;
    elsif v_purchase_id is not null and coalesce(v_purchase_amount_minor, 0) > 0 then
        insert into event_refund_request (
            event_purchase_id,
            requested_by_user_id,
            requested_reason,
            status
        )
        select
            v_purchase_id,
            p_actor_user_id,
            'Attendance canceled by organizer',
            'pending'
        where not exists (
            select 1
            from event_refund_request err
            where err.event_purchase_id = v_purchase_id
            and err.status in ('pending', 'approving')
        );

        update event_purchase
        set
            status = 'refund-requested',
            updated_at = current_timestamp
        where event_purchase_id = v_purchase_id
        and status = 'completed';
    end if;

    -- Promote the next waitlisted user when a confirmed attendee frees a seat
    if not v_is_ticketed then
        select promote_event_waitlist(
            p_event_id,
            case when v_capacity is null then null else 1 end
        )
        into v_promoted_user_ids;
    end if;

    -- Track the cancellation
    perform insert_audit_log(
        'event_attendee_attendance_canceled',
        p_actor_user_id,
        'user',
        p_user_id,
        v_alliance_id,
        p_group_id,
        p_event_id,
        jsonb_build_object('event_id', p_event_id, 'user_id', p_user_id)
    );

    return json_strip_nulls(json_build_object(
        'left_status', 'attendee',
        'promoted_user_ids', v_promoted_user_ids,
        'refund_event_purchase_id', case
            when coalesce(v_purchase_amount_minor, 0) > 0 then v_purchase_id
            else null
        end
    ));
end;
$$ language plpgsql;
