# Security policy

CubeSat Classroom is a work-in-progress education project. There are no supported releases yet; fixes land on `main`.

## Reporting a vulnerability

Please do not open a public issue for security problems. Use GitHub's private reporting instead: **Security > Report a vulnerability** on this repository. Include what you found, how to reproduce it and what it affects. You should get a reply within a week.

In scope: the Rust ground station, the Cloudflare Worker in `site/src/worker.mjs`, the installer (`docs/start.sh`), the mobile app and the CI workflows.

## Handling secrets and personal data

- Never commit API tokens, keys, passwords or key files. Deployment secrets live only in repository or Worker secrets.
- Never commit student names, schools, locations, photos or contact details. Use team nicknames in crew logs.
- If a secret is committed by mistake, treat it as exposed: revoke and replace it first, then remove it from the repository.
