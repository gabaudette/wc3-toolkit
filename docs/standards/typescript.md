# TypeScript

General AGE standards continue to apply to TypeScript projects.

## AGE-TS-001 — Prefer `type` over `interface`

Use `type` by default for named object shapes and other TypeScript types.

```ts
type Player = {
  id: string;
  name: string;
  health: number;
};
```

Use `interface` when interface-specific semantics are intentionally required, such as declaration merging or framework-required augmentation.

Do not choose `interface` merely because a type describes an object.

## AGE-TS-002 — Biome is the standard toolchain

AGE uses Biome for TypeScript and JavaScript formatting, linting, and import organization.

Do not introduce ESLint or Prettier unless a project has a specific technical requirement Biome cannot reasonably satisfy.

The Biome schema version may change as tooling is upgraded.

### AGE baseline

```json
{
  "$schema": "https://biomejs.dev/schemas/2.5.14/schema.json",
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentWidth": 2,
    "lineWidth": 100
  },
  "linter": {
    "enabled": true,
    "rules": {
      "preset": "recommended",
      "a11y": {
        "useKeyWithClickEvents": "off",
        "noStaticElementInteractions": "off",
        "noLabelWithoutControl": "off",
        "noSvgWithoutTitle": "off"
      },
      "complexity": {
        "noForEach": "error"
      },
      "style": {
        "useBlockStatements": "error",
        "noYodaExpression": "error",
        "noSubstr": "error",
        "noMagicNumbers": "error",
        "noTernary": "error",
        "noEnum": "error",
        "useNumericSeparators": "error"
      },
      "suspicious": {
        "noConsole": "error",
        "noExplicitAny": "error"
      }
    }
  },
  "javascript": {
    "formatter": {
      "quoteStyle": "double",
      "trailingCommas": "all",
      "semicolons": "always"
    }
  },
  "css": {
    "parser": {
      "tailwindDirectives": true
    }
  },
  "assist": {
    "enabled": true,
    "actions": {
      "source": {
        "organizeImports": "on"
      }
    }
  }
}
```

Project-specific file exclusions and overrides may be added when justified by the domain, but they should be narrow.

Examples include rendering code where numeric literals are intrinsic or one-off migration scripts where direct terminal output is intentional.

### Additional TypeScript conventions

The baseline enforces several AGE preferences:

- block statements for control flow;
- no `forEach`;
- no ternary expressions by default;
- no TypeScript `enum`;
- no explicit `any`;
- no unexplained magic numbers;
- numeric separators for large numeric literals;
- no direct `console` usage in application code.

Operational output should use the project's logging and monitoring infrastructure.
