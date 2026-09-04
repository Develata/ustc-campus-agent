# Security Policy

## Current status

USTC Campus Agent is a public, MIT-licensed student competition prototype. It is not an official USTC service and must not be used for production access to USTC systems.

## Sensitive data rules

Never commit:

- USTC unified identity credentials, CAS tickets, cookies, MFA material, or screenshots exposing them;
- API keys, GitHub tokens, model provider keys, private keys, or `.env` files;
- real student academic snapshots, grades, ranking, phone numbers, or identifying course-plan exports;
- raw iCourse review corpora unless explicit permission and a data contract exist.

## Reporting

Do not put credentials, private student data, exploit details, or proof-of-concept payloads in a public issue.

Use GitHub's **Report a vulnerability** action on the repository Security page when it is available. If that private channel is unavailable, open a public issue containing only the title `[security contact request]` and a request for a private maintainer channel; do not include vulnerability details. Maintainers will acknowledge the safe contact request, coordinate a private report, and publish a security advisory when disclosure is appropriate. No response-time SLA is claimed for this competition prototype.
