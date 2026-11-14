-- Returns the current enrollment counts of an event.
--
-- Registrations are attributed to the path that admitted them: the source of
-- the admission offer claimed by the purchase holding the seat, a direct
-- purchase, or a manual invitation recorded without a purchase.
create or replace function stats_event_enrollment(p_event event)
returns json as $$
with attendees as (
    select
        count(*) filter (where ea.status = 'attendance-canceled')::bigint as cancellations,
        count(*) filter (where ea.status = 'confirmed' and ea.checked_in)::bigint as checked_in,
        count(*) filter (where ea.status = 'confirmed')::bigint as registrations
    from event_attendee ea
    where ea.event_id = p_event.event_id
),
invitation_requests as (
    select
        count(*) filter (where eir.status = 'accepted')::bigint as accepted,
        count(*) filter (where eir.status = 'pending')::bigint as pending,
        count(*) filter (where eir.status = 'rejected')::bigint as rejected
    from event_invitation_request eir
    where eir.event_id = p_event.event_id
),
seat_purchases as (
    select distinct on (ep.user_id)
        ep.user_id
    from event_purchase ep
    where ep.event_id = p_event.event_id
    and event_purchase_holds_seat(ep.status)
    order by
        ep.user_id,
        ep.completed_at desc nulls last,
        ep.created_at desc,
        ep.event_purchase_id desc
),
registration_paths as (
    select
        case
            when sp.user_id is not null then 'direct'
            when ea.manually_invited then 'organizer_invitation'
            else 'unknown'
        end as path
    from event_attendee ea
    left join seat_purchases sp on sp.user_id = ea.user_id
    where ea.event_id = p_event.event_id
    and ea.status = 'confirmed'
)
select json_build_object(
    'admission_offers', '[]'::json,
    'cancellations', a.cancellations,
    'checked_in', a.checked_in,
    'invitation_requests', json_build_object(
        'accepted', ir.accepted,
        'pending', ir.pending,
        'rejected', ir.rejected
    ),
    'registrations', a.registrations,
    'registrations_by_path', (
        select json_build_object(
            'approval', count(*) filter (where rp.path = 'approval'),
            'direct', count(*) filter (where rp.path = 'direct'),
            'organizer_invitation', count(*) filter (where rp.path = 'organizer_invitation'),
            'unknown', count(*) filter (where rp.path = 'unknown'),
            'waitlist', count(*) filter (where rp.path = 'waitlist')
        )
        from registration_paths rp
    ),
    'waitlist', (
        select count(*)::bigint
        from event_waitlist ew
        where ew.event_id = p_event.event_id
    )
)
from attendees a
cross join invitation_requests ir;
$$ language sql stable;
