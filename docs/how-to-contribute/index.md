# How to contribute

Welcome to the Open Alliance Groups (GOUP) project. Contributions are accepted via GitHub pull requests. This page covers how to pick up work, open a PR, and meet the definition of done.

## Finding work

- Browse open issues at <https://github.com/goup/open-alliance-groups/issues>.
- Start a discussion before investing significant effort on a new feature — use <https://github.com/goup/open-alliance-groups/discussions>.
- If you want to add a third-party service integration, read `docs/guides/integrations.md` first.

## Setting up your environment

Before writing any code, follow the steps in [Development workflow](development-workflow.md) to bring up a local database, apply migrations, and confirm the test suites pass.

## Branch and commit conventions

- Create a feature or fix branch from `main`, e.g. `feat/my-feature` or `fix/issue-123`.
- Every commit must carry a DCO sign-off (see `CONTRIBUTING.md`):

  ```
  git commit -s -m "feat: describe the change"
  ```

  The `Author` and `Signed-off-by` e-mail addresses must match.

## Opening a pull request

1. Push your branch and open a PR against `main`.
2. Fill in the PR description: what changed, why, and how to test it.
3. If you are unsure whether a change will be accepted, open an issue first to get feedback.
4. The CI pipeline (`.github/workflows/ci.yml`) runs automatically on every PR. All checks must be green before merge.
5. For visual changes, update Playwright snapshots with `just e2e-update-snapshots` and commit the diff.

## Review expectations

- At least one maintainer approval is required.
- Address every review comment before requesting a re-review. If you disagree with a comment, explain why in the thread.
- Avoid force-pushing after a review has started; add fixup commits instead so the diff is easy to follow.

## Definition of done

A PR is ready to merge when:

- All CI checks pass (lint, format, unit tests, database tests, contract tests).
- The reviewer has approved the PR.
- Every commit is signed off.
- New behaviour is covered by tests (Rust unit/integration, pgTAP, or Playwright as appropriate).
- Documentation is updated if the change affects public-facing behaviour or configuration.
