-- Returns the number of seats allocated for an event ticket type.
create or replace function get_event_ticket_type_allocated_seat_count(
    p_event_id uuid,
    p_event_ticket_type_id uuid
)
returns int as $$
    select count(*)::int
    from event_purchase ep
    where ep.event_id = p_event_id
    and ep.event_ticket_type_id = p_event_ticket_type_id
    and event_purchase_reserves_seat(ep.status, ep.hold_expires_at);
$$ language sql stable;
