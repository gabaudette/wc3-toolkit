# Code Quality

## AGE-QUAL-001 — Never ignore diagnostics

Compiler, linter, type-checker, and configured static-analysis diagnostics must be addressed.

The expected state is:

```text
Errors:   0
Warnings: 0
```

## AGE-QUAL-002 — Warnings are errors

Where supported, warnings must be promoted to validation failures.

Warnings do not accumulate as accepted technical debt.

## AGE-QUAL-003 — Fix the cause

Resolve the underlying problem rather than silencing the diagnostic.

Narrow suppressions are acceptable only for genuine false positives, unavoidable external constraints, or equivalent technical reasons. They must be documented and scoped as tightly as possible.

## AGE-QUAL-004 — Do not weaken tooling to pass CI

Do not relax compiler, linter, formatter, type-checker, or static-analysis configuration merely to make failing code pass.

A rule may be changed when it is genuinely inappropriate for the project, but the decision must be intentional rather than a workaround.

## AGE-QUAL-005 — CI enforces a clean build

Where applicable, CI should validate:

```text
Source
  ├── Formatter
  ├── Linter
  ├── Type checker
  ├── Compiler
  └── Tests
       ↓
    Mergeable
```

## AGE-QUAL-006 — Use the language's standard formatter

When a language provides an official, canonical, or clearly established standard formatter, AGE uses it.

Examples include:

```text
Go       → gofmt
Rust     → rustfmt
Odin     → odinfmt
```

The formatter owns formatter-controlled formatting. AGE aesthetic preferences do not override it.

TypeScript and JavaScript use Biome as defined by the TypeScript standard.

If a language has no standard formatter, select one based on ecosystem adoption, maintenance, deterministic output, and tooling integration.

## AGE-QUAL-007 — IDE and editor agnostic

AGE does not mandate an IDE or code editor.

Developers may use any suitable environment, but it must integrate with the tooling defined by the project:

- formatter;
- linter;
- LSP or equivalent language tooling;
- compiler diagnostics;
- type checker;
- import organization;
- static analysis.

The repository defines the standard. The editor integrates with it. CI enforces it.

Important project behavior must not exist only as an editor-specific inspection or local setting.
