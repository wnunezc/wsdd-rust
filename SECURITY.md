# Security Policy

## Supported versions

Security fixes are only guaranteed for the latest release candidate published in the `main`
branch and for subsequent versions under active maintenance.

At this moment, the actively maintained line is:

| Version line | Supported |
|---|---|
| `1.0.0-rc.18` and newer | Yes |
| Older release candidates | No |

## Reporting a vulnerability

Please **do not** open a public GitHub Issue for security-sensitive reports.

Preferred reporting channels:

1. GitHub Security Advisories private reporting, if available for this repository
2. Email: [wnunez@lh-2.net](mailto:wnunez@lh-2.net)

When reporting a vulnerability, include:

- A short summary of the issue
- Affected version or commit
- Reproduction steps or proof of concept
- Impact assessment
- Any suggested remediation, if available

## Response expectations

The maintainer will try to:

- Confirm receipt of the report
- Assess severity and reproducibility
- Coordinate a fix before public disclosure when appropriate

Response times may vary depending on availability, but private reports will be prioritized over
public discussion.

## Scope notes

WSDD is a Windows desktop application that automates local development infrastructure. Reports
are especially useful when they involve:

- Privilege escalation paths
- Unsafe PowerShell execution
- Credential or secret exposure
- Insecure update or package distribution flows
- Docker or filesystem actions that could escape intended paths
