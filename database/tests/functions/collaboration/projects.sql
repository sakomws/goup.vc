begin;
select plan(20);

select has_table('collaboration_project');
select has_table('collaboration_project_member');
select has_table('collaboration_project_goal');
select has_table('collaboration_project_task');
select has_table('collaboration_project_update');
select has_table('collaboration_project_activity');
select has_table('collaboration_project_outcome');
select has_table('collaboration_project_evidence');
select has_table('collaboration_office_hour_session');
select has_table('collaboration_office_hour_booking');
select has_table('collaboration_notification_delivery');

select has_function('collaboration_project_json', array['collaboration_project']);
select has_function('get_collaboration_outcome_summary', array['uuid']);
select has_function('create_collaboration_project', array['uuid', 'uuid', 'jsonb']);
select has_function('get_group_collaboration_dashboard', array['uuid']);
select has_function('get_public_collaboration_project', array['uuid', 'text', 'text']);
select has_function('submit_collaboration_update', array['uuid', 'uuid', 'text', 'text', 'text']);
select has_function('book_collaboration_office_hour', array['uuid', 'uuid', 'text']);
select has_function('enqueue_due_collaboration_reminders', array['timestamp with time zone']);

select results_eq(
    $$select display_name from group_permission where group_permission_id = 'group.projects.write'$$,
    $$values ('Projects Write'::text)$$,
    'project management permission is registered'
);

select * from finish();
rollback;
