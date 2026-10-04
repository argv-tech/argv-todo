# Security policy

## Supported versions

Security fixes target the latest published release. Before the first release, fixes target the `main` branch. Older versions may not receive backports.

## Reporting a vulnerability

Please report suspected security vulnerabilities privately. Do not include exploit details, credentials, or personal task data in a public issue.

If private vulnerability reporting is enabled, use [Report a vulnerability](https://github.com/argv-tech/argv-todo/security/advisories/new) in the repository's Security tab.

If that option is unavailable, use a private contact method listed on a maintainer's GitHub profile. If no private contact is listed, open an issue asking for a security contact, without describing the vulnerability.

Include the following in a private report:

- The affected version or commit, operating system, and terminal.
- A description of the issue and its potential impact.
- Reproduction steps using a temporary database and fictional task data.
- Any proposed fix or workaround.

Maintainers will investigate and coordinate a fix and disclosure as availability allows. There is no guaranteed response time or bug bounty.

## Local data

Tasks are stored in a local SQLite database. The database is not encrypted by the application. Anyone with access to the file may be able to read or change its contents; protect it using your operating system's permissions and storage protections.

Do not attach your personal database to an issue. Reproduce problems with sample data instead.
