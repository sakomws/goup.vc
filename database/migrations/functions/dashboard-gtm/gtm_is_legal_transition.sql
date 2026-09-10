-- Agents may only suggest the next legal stage. Humans may move to any stage.
create or replace function gtm_is_legal_transition(
    p_from text,
    p_to text,
    p_human boolean
) returns boolean language sql immutable as $$
    select case
        when p_to is null or p_from is null then false
        when p_from = p_to then true
        when coalesce(p_human, false) then true
        when p_from = 'lead_generation' and p_to = 'reachout' then true
        when p_from = 'reachout' and p_to = 'get_response' then true
        when p_from = 'get_response' and p_to = 'qualification' then true
        when p_from = 'qualification' and p_to in ('proposal', 'lost') then true
        when p_from = 'proposal' and p_to = 'negotiation' then true
        when p_from = 'negotiation' and p_to in ('won', 'lost') then true
        when p_from = 'won' and p_to = 'delivered' then true
        when p_from = 'delivered' and p_to = 'renewal' then true
        when p_from = 'renewal' and p_to = 'reachout' then true
        when p_from = 'lost' and p_to = 'reachout' then true
        else false
    end;
$$;
