# Whitespace

Whitespace should communicate the logical structure of code.

Use exactly one blank line when separating logical units unless the language's authoritative formatter requires otherwise.

## AGE-WS-001 — Block separation

A completed block at the same syntactic level as the following statement or block should be separated from it by one blank line.

### Preferred

```text
if condition {
    do_something();
}

do_something_else();
```

Do not insert a blank line before syntax that continues the same construct.

```text
if condition {
    do_something();
} else {
    do_something_else();
}
```

The same principle applies to constructs such as `catch`, `finally`, and equivalent language-specific continuation syntax.

Nested closing braces do not require blank lines solely because a block ended.

## AGE-WS-002 — Logical declaration groups

Variable declarations belonging to the same logical purpose should remain contiguous.

Different logical declaration groups should be separated by one blank line. The final declaration group should also be separated from subsequent executable code.

### Preferred

```text
let width = 1280;
let height = 720;

let player = get_player();
let enemy = get_enemy();

start_game(width, height, player, enemy);
```

Do not separate every declaration mechanically. Blank lines should communicate a meaningful change in purpose.

## Formatter precedence

AGE whitespace rules apply where they do not conflict with the project's authoritative formatter.

When an official language formatter or an AGE-defined formatter makes a different formatting decision, the formatter wins.
