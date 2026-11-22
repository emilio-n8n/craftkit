# Security policy

CraftKit is a local desktop foundation with no network service. Report a
vulnerability privately to the maintainers rather than opening a public issue
when it could affect users of downstream applications.

## Supported versions

The latest minor release receives security fixes.

## Scope

- Unsafe parsing of project files or external assets.
- Path traversal during import or export.
- Command injection through external converters.
- Dependency vulnerabilities with a reachable impact.

Please include the affected version, a minimal reproduction, and the expected
impact.
