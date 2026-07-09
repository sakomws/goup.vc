<!-- markdownlint-disable MD013 -->

# Jobs and Mock Interviews

Use this guide to browse jobs, post opportunities, request mock interview practice, and manage the
mock interview queue.

Paths:

- Public jobs board: [/jobs](/jobs ':ignore')
- Public mock interviews page: [/jobs/mock-interviews](/jobs/mock-interviews ':ignore')
- Jobs dashboard: [/dashboard/jobs](/dashboard/jobs ':ignore')
- Mock interviews dashboard: [/dashboard/jobs/mock-interviews](/dashboard/jobs/mock-interviews ':ignore')

## Public Jobs Board

The public jobs board lists roles shared by GOUP members, partners, and open-source teams.

Visitors can:

- Search by title, company, summary, description, or tags.
- Filter by location.
- Filter for remote-friendly roles.
- Open a role details page.

Logged-in members can also save interest on a role. The job poster can review saved interest from
the Jobs dashboard and follow up directly.

## Posting Jobs

Logged-in members can post roles from [Jobs Dashboard](/dashboard/jobs ':ignore').

Job posters can:

- Create a draft role with title, company, summary, description, location, apply URL, and tags.
- Mark the role as remote-friendly.
- Choose public visibility or members-only visibility.
- Publish, unpublish, edit, or delete roles.
- Review saved interest with member profile context and contact links.

Use public visibility for roles anyone can apply to. Use members-only visibility when the role should
only be shown to logged-in GOUP members.

## Mock Interview Practice

Mock interviews live under the Jobs product area because they help members prepare for job searches,
founder interviews, and technical screens.

The public page uses poll-informed categories so practice requests are easy to match:

- Practice role: interviewee, interviewer, both, or not interested.
- Interview type: software engineering, AI/ML, startup or co-founder, product management,
  DevOps/cloud, security, behavioral/HR, or other.
- Target company: remote/global companies, AI labs or FAANG, enterprise, AI startup, or flexible.
- Seniority: graduate/junior, mid, senior, or staff-plus.
- Location: AZE, USA/Canada, EU, Asia, or other.

Logged-in members can join the practice queue from
[/jobs/mock-interviews](/jobs/mock-interviews ':ignore') by selecting those categories and adding
availability notes.

## Scheduling Workflow

Organizers use [Mock Interviews Dashboard](/dashboard/jobs/mock-interviews ':ignore') to turn
requests into scheduled sessions.

The dashboard supports:

- Filtering requests by status.
- Reviewing requester profile, email, availability, and notes.
- Assigning interviewer and interviewee user IDs.
- Setting a scheduled time.
- Adding a meeting URL.
- Tracking status as matched, scheduled, completed, or canceled.
- Recording interviewer and interviewee feedback after the session.

The current implementation keeps one match per request. Updating a request's match replaces the
stored schedule details for that request.

## Data Model

Mock interviews use two tables:

- `mock_interview_request`: one row per member request. It stores the requester, poll category
  choices, availability, notes, request status, and timestamps.
- `mock_interview_match`: one row per scheduled/matched request. It stores assigned users,
  scheduled time, meeting URL, match status, internal notes, feedback, and timestamps.

The request status mirrors the match status after scheduling actions so the queue can be filtered
without joining match data.

## Contributor Notes

When changing this area, check these files first:

- `database/migrations/schema/0040_mock_interviews.sql`
- `ocg-server/src/types/mock_interviews.rs`
- `ocg-server/src/db/mock_interviews.rs`
- `ocg-server/src/handlers/site/jobs.rs`
- `ocg-server/src/handlers/dashboard/jobs.rs`
- `ocg-server/templates/site/jobs/mock_interviews.html`
- `ocg-server/templates/dashboard/jobs/mock_interviews.html`

Recommended checks:

```bash
cargo check -p ocg-server
cargo test -p ocg-server mock_interviews
just db-tests
uvx --python 3.12 djlint==1.39.2 --check --configuration ocg-server/templates/.djlintrc ocg-server/templates
```
