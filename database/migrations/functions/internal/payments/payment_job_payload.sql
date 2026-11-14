-- Builds the kind-specific provider context a worker needs for a claimed payment job.
create or replace function payment_job_payload(p_job payment_job)
returns jsonb as $$
declare
    v_alliance_id uuid;
    v_event_id uuid;
    v_purchase event_purchase;
    v_refund event_purchase_refund;
begin
    -- Load the purchase every payment job hangs off
    select ep.*
    into v_purchase
    from event_purchase ep
    where ep.event_purchase_id = p_job.event_purchase_id;

    -- Reject jobs whose purchase disappeared
    if not found then
        raise exception 'event purchase not found';
    end if;

    -- Build the payload of the job kind
    case p_job.kind
        -- Refunds return the provider payment and the notification context
        when 'event-purchase-refund' then
            select epr.*
            into v_refund
            from event_purchase_refund epr
            where epr.payment_job_id = p_job.payment_job_id;

            -- Reject jobs without their refund row
            if not found then
                raise exception 'event purchase refund not found';
            end if;

            -- Resolve the alliance and event the notification is composed for
            select g.alliance_id, e.event_id
            into v_alliance_id, v_event_id
            from event e
            join "group" g on g.group_id = e.group_id
            where e.event_id = v_purchase.event_id;

            return jsonb_build_object(
                'refund', event_purchase_refund_to_json(v_refund, p_job)
                    || jsonb_strip_nulls(jsonb_build_object(
                        'alliance_id', v_alliance_id,
                        'event_id', v_event_id,
                        'provider_payment_reference', v_purchase.provider_payment_reference
                    ))
            );

        -- Reject kinds without a worker payload
        else
            raise exception 'payment job kind has no worker payload';
    end case;
end;
$$ language plpgsql;
