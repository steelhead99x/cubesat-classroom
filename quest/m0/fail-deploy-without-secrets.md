# [S] Fail the Worker deploy when Cloudflare secrets are missing

## Goal

The Deploy Worker workflow exits non-zero when `CLOUDFLARE_API_TOKEN` or `CLOUDFLARE_ACCOUNT_ID` is unset, so a missing secret cannot look like a successful deploy. Pull requests and forks are unaffected.

## Plan

1. Open `.github/workflows/deploy-worker.yml`. Confirm the "Cloudflare secrets missing" step runs `exit 1` (and an `::error::` message) when `HAS_CLOUDFLARE` is not true, instead of a notice that leaves the job green.
2. Confirm the workflow `on:` block is only `workflow_run` of CI on `main` and `workflow_dispatch`. There is no `pull_request` trigger, so fork PRs and branch CI do not run this job.
3. Confirm the job `if:` still requires a successful CI `push` to main, or a manual start.
4. Until the maintainer adds both repository secrets, a push to main that passes CI will leave Deploy Worker red. That is intended. After the secrets are added, re-run the workflow and confirm it deploys.

### How to check

- [ ] The missing-secrets step contains `exit 1`.
- [ ] The workflow does not trigger on `pull_request`.
- [ ] `quest check` from the repository root still passes.

## Required

- [Teach the two-relay lesson](/quest/m0/repeat-the-station-lesson.md) - the public site is how a class finds the station installer.
