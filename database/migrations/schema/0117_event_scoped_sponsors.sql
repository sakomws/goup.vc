-- Allow sponsors to belong to one event without entering the reusable group catalog.

alter table group_sponsor
    add column if not exists event_id uuid references event (event_id);

create index if not exists group_sponsor_event_id_idx
    on group_sponsor (event_id)
    where event_id is not null;

create or replace function check_group_sponsor_event_scope()
returns trigger as $$
declare
    v_event_group_id uuid;
begin
    if new.event_id is null then
        return new;
    end if;

    select group_id
    into v_event_group_id
    from event
    where event_id = new.event_id;

    if v_event_group_id is null then
        raise exception 'event not found';
    end if;

    if new.group_id is distinct from v_event_group_id then
        raise exception 'event sponsor must belong to the event group';
    end if;

    new.featured := false;
    return new;
end;
$$ language plpgsql;

drop trigger if exists group_sponsor_event_scope_check on group_sponsor;
create trigger group_sponsor_event_scope_check
before insert or update of event_id, group_id, featured on group_sponsor
for each row
execute function check_group_sponsor_event_scope();

create or replace function check_event_sponsor_group()
returns trigger as $$
declare
    v_event_group_id uuid;
    v_sponsor_event_id uuid;
    v_sponsor_group_id uuid;
begin
    select group_id
    into v_event_group_id
    from event
    where event_id = new.event_id;

    select event_id, group_id
    into v_sponsor_event_id, v_sponsor_group_id
    from group_sponsor
    where group_sponsor_id = new.group_sponsor_id;

    if v_sponsor_group_id is distinct from v_event_group_id then
        raise exception 'sponsor not found in group';
    end if;

    if v_sponsor_event_id is not null
       and v_sponsor_event_id is distinct from new.event_id then
        raise exception 'sponsor belongs to a different event';
    end if;

    return new;
end;
$$ language plpgsql;
