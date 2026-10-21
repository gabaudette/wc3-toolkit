# Technology Policy

## AGE-TECH-001 — Language and framework agnostic

AGE does not mandate one language or framework across all projects.

Technology should be chosen according to the project's requirements, maintainability, ecosystem, tooling, performance, deployment environment, interoperability, and long-term support.

## AGE-TECH-002 — Avoid technology loyalty

Previous use is a legitimate consideration, not an automatic decision.

AGE should neither choose a technology only because it is familiar nor reject an appropriate technology simply because the studio has not used it before.

## AGE-TECH-003 — Prohibited languages

The following languages must not be introduced into new AGE-owned production code:

- Ruby
- Objective-C
- Visual Basic
- Pascal
- Fortran
- Perl
- Lisp-family languages
- Haskell

This is an AGE engineering policy, not a claim that these languages are technically incapable or universally obsolete.

Interoperating with legacy systems, dependencies, vendor systems, or migration targets written in these languages is allowed. New AGE-owned development should not deepen dependency on them.

## AGE-TECH-004 — Open source first

AGE strongly favors open-source software.

When two solutions satisfy project requirements reasonably well, prefer the open-source option.

Technology evaluation should consider:

- control and inspectability;
- portability;
- interoperability;
- ability to self-host;
- vendor independence;
- data export and migration;
- open standards;
- licensing compatibility.

Proprietary technology may be used when required by a target platform or when it provides a substantial advantage that reasonable open alternatives cannot provide.

Necessary proprietary dependencies should be isolated behind AGE-owned boundaries when practical.

## AGE-TECH-005 — Prefer platform-agnostic code

AGE code and general development tooling should be operating-system agnostic by default.

Do not write OS-specific code merely for convenience.

A general project setup script should not be PowerShell-only. Replacing it with Bash-only tooling does not solve the portability issue.

Prefer:

- portable language-native tooling;
- cross-platform task runners;
- project CLIs;
- platform-independent filesystem and path APIs.

Platform-specific code is appropriate when the platform itself is intentionally being targeted, such as OS integration, platform packaging, vendor SDKs, or platform-specific hardware features.

When platform-specific behavior is necessary in an otherwise portable application, isolate it behind a common boundary.

**Platform-specific by requirement, not by convenience.**
