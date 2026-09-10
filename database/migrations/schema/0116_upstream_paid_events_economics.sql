-- Port CNCF paid-event economics onto GOUP's existing checkout:
-- platform fee, tax/invoice snapshots, external payments, parent groups.

alter table event_purchase
    add column if not exists platform_fee_amount_minor bigint default 0 not null,
    add column if not exists charge_model text default 'stripe' not null,
    add column if not exists tax_amount_minor bigint,
    add column if not exists tax_behavior text,
    add column if not exists tax_calculation_mode text,
    add column if not exists provider_invoice_id text,
    add column if not exists provider_invoice_hosted_url text,
    add column if not exists provider_invoice_pdf_url text,
    add column if not exists external_payment_details text,
    add column if not exists external_payment_marked_by_user_id uuid references "user" (user_id) on delete set null;

do $$
begin
    if not exists (
        select 1
        from pg_constraint
        where conname = 'event_purchase_platform_fee_amount_minor_chk'
    ) then
        alter table event_purchase
            add constraint event_purchase_platform_fee_amount_minor_chk check (
                platform_fee_amount_minor >= 0
                and platform_fee_amount_minor <= amount_minor
            );
    end if;

    if not exists (
        select 1
        from pg_constraint
        where conname = 'event_purchase_charge_model_chk'
    ) then
        alter table event_purchase
            add constraint event_purchase_charge_model_chk check (
                charge_model in ('stripe', 'external', 'free')
            );
    end if;
end;
$$;

alter table event
    add column if not exists tax_behavior text default 'inclusive' not null,
    add column if not exists tax_calculation_mode text default 'none' not null,
    add column if not exists external_payment_instructions text,
    add column if not exists external_payment_url text,
    add column if not exists external_payment_window_hours integer;

do $$
begin
    if not exists (
        select 1
        from pg_constraint
        where conname = 'event_tax_behavior_chk'
    ) then
        alter table event
            add constraint event_tax_behavior_chk check (
                tax_behavior in ('exclusive', 'inclusive')
            );
    end if;

    if not exists (
        select 1
        from pg_constraint
        where conname = 'event_tax_calculation_mode_chk'
    ) then
        alter table event
            add constraint event_tax_calculation_mode_chk check (
                tax_calculation_mode in ('automatic', 'manual', 'none')
            );
    end if;
end;
$$;

alter table "group"
    add column if not exists external_payments_enabled boolean not null default false,
    add column if not exists parent_group_id uuid references "group" (group_id);

create index if not exists group_parent_group_id_idx on "group" (parent_group_id);

create table if not exists external_payments_config (
    singleton boolean primary key default true check (singleton),
    allowed_countries text[] not null default '{}'::text[],
    default_payment_window_hours integer not null default 72,
    max_payment_window_hours integer not null default 336,
    updated_at timestamp with time zone default current_timestamp not null,
    constraint external_payments_config_window_hours_chk check (
        default_payment_window_hours >= 1
        and max_payment_window_hours >= default_payment_window_hours
        and max_payment_window_hours <= 8760
    )
);

insert into external_payments_config (
    singleton,
    allowed_countries,
    default_payment_window_hours,
    max_payment_window_hours
) values (true, '{}'::text[], 72, 336)
on conflict (singleton) do nothing;

create or replace function check_group_parent_relationship()
returns trigger as $$
declare
    v_parent record;
begin
    if new.parent_group_id is null then
        return new;
    end if;

    if new.parent_group_id = new.group_id then
        raise exception 'group cannot be its own parent';
    end if;

    if tg_op = 'UPDATE'
       and new.parent_group_id is not distinct from old.parent_group_id then
        return new;
    end if;

    select
        g.active,
        g.alliance_id,
        g.deleted,
        g.group_id,
        g.parent_group_id
    into v_parent
    from "group" g
    where g.group_id = new.parent_group_id
    for update;

    if not found then
        raise exception 'parent group not found';
    end if;

    if v_parent.alliance_id <> new.alliance_id then
        raise exception 'parent group must belong to the same alliance';
    end if;

    if v_parent.deleted then
        raise exception 'parent group cannot be deleted';
    end if;

    if not v_parent.active then
        raise exception 'parent group must be active';
    end if;

    if v_parent.parent_group_id is not null then
        raise exception 'parent group cannot be a subgroup';
    end if;

    if exists (
        select 1
        from "group" child
        where child.parent_group_id = new.group_id
          and child.deleted = false
          and child.group_id <> new.group_id
    ) then
        raise exception 'group with subgroups cannot have a parent';
    end if;

    return new;
end;
$$ language plpgsql;

drop trigger if exists group_parent_relationship_check on "group";
create trigger group_parent_relationship_check
before insert or update of parent_group_id on "group"
for each row
execute function check_group_parent_relationship();

insert into notification_kind (name, optional_notification)
values
    ('event-external-payment-expired', false),
    ('event-external-payment-pending', false),
    ('event-external-payment-reminder', false)
on conflict (name) do nothing;
