begin;
select plan(15);

select has_table('event_sponsor_engagement_daily');
select has_table('event_sponsor_manual_engagement');
select has_table('event_sponsor_report_share');
select has_pk('event_sponsor_engagement_daily');
select has_pk('event_sponsor_manual_engagement');
select has_pk('event_sponsor_report_share');
select fk_ok(
    'event_sponsor_engagement_daily',
    array['group_sponsor_id', 'event_id']::name[],
    'event_sponsor',
    array['group_sponsor_id', 'event_id']::name[]
);
select fk_ok(
    'event_sponsor_manual_engagement',
    array['group_sponsor_id', 'event_id']::name[],
    'event_sponsor',
    array['group_sponsor_id', 'event_id']::name[]
);
select has_function(
    'record_event_sponsor_engagement',
    array['uuid', 'uuid', 'text', 'uuid']::name[]
);
select has_function('get_event_sponsor_report', array['uuid']::name[]);
select has_function('get_public_event_sponsor_report', array['uuid']::name[]);
select has_function(
    'update_event_sponsor_manual_engagement',
    array['uuid', 'uuid', 'uuid', 'uuid', 'jsonb']::name[]
);
select has_function(
    'create_event_sponsor_report_share',
    array['uuid', 'uuid', 'uuid']::name[]
);
select has_function(
    'revoke_event_sponsor_report_share',
    array['uuid', 'uuid']::name[]
);
select col_is_unique('event_sponsor_report_share', 'token');

select * from finish();
rollback;
