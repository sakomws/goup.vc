begin;
select plan(10);

select has_table('event_registration_attribution');
select has_table('event_finance_entry');
select has_pk('event_registration_attribution');
select has_pk('event_finance_entry');
select col_is_fk('event_registration_attribution', 'event_id', 'event');
select col_is_fk('event_registration_attribution', 'user_id', 'user');
select has_function(
    'capture_event_registration_attribution',
    array['uuid', 'uuid', 'jsonb']::name[]
);
select has_function('get_event_growth', array['uuid', 'uuid']::name[]);
select has_function(
    'add_event_finance_entry',
    array['uuid', 'uuid', 'uuid', 'jsonb']::name[]
);
select has_function(
    'delete_event_finance_entry',
    array['uuid', 'uuid', 'uuid']::name[]
);

select * from finish();
rollback;
