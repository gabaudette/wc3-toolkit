# Error Handling

AGE follows four core principles:

> Errors are values.

> Exceptions are contained at boundaries.

> A crash is a last resort.

> Handled does not mean invisible.

## AGE-ERR-001 — Errors are values

Recoverable failures must be represented explicitly through return values.

Prefer mechanisms such as:

- `Result<T, E>`
- `(T, error)`
- tagged or discriminated unions
- explicit multiple return values
- language-specific equivalents

Fallible AGE APIs should make failure visible in their contract.

## AGE-ERR-002 — Exceptions must be contained

When a language, framework, standard library, or dependency throws exceptions, catch them at the appropriate boundary and convert them to AGE's normal error representation.

```text
exception-based dependency
        ↓
      catch
        ↓
convert + preserve context
        ↓
   Result<T, E>
        ↓
    AGE code
```

Exceptions should not propagate through AGE application code merely because an external dependency uses them.

Catch narrowly. Exception handling is an interoperability mechanism, not normal AGE control flow.

## AGE-ERR-003 — Crashing is a last resort

An error must not terminate the application when the program can reasonably:

- recover;
- retry;
- degrade functionality;
- return to a safe state;
- report the problem to the user;
- return the error to its caller.

Termination is reserved for situations where continuing is impossible, unsafe, or risks further corruption.

When termination is necessary, prefer controlled shutdown with diagnostic reporting over an arbitrary unhandled exception or panic.

Assertions are appropriate for programmer errors and violated invariants. They are not a substitute for runtime error handling.

## AGE-ERR-004 — Failures must be observable

Operationally relevant failures must reach the project's monitoring and observability tooling.

Depending on the project, this may include:

- structured logs;
- error reporting;
- metrics;
- traces;
- alerts.

Expected user mistakes or routine absence conditions do not automatically require an error event. Use the appropriate level and avoid flooding monitoring systems.

### Report once

Do not report the same failure independently at every layer.

Prefer returning and contextualizing the error until it reaches a layer with enough information to understand its operational significance, then report it once.

### Preserve context

Errors should retain enough context to diagnose the original problem without leaking sensitive data.
