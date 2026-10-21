# Design Principles

## AGE-DES-001 — Prefer composition over inheritance

Composition is AGE's default approach to code reuse and object modeling.

Prefer building types from focused capabilities:

```text
Player
├── Health
├── Inventory
├── Movement
└── Combat
```

rather than creating deep inheritance hierarchies solely to reuse behavior.

Inheritance is not forbidden. It can be appropriate when:

- a genuine and stable is-a relationship exists;
- a framework or platform requires a base class;
- a small hierarchy accurately represents the domain;
- inheritance is clearly simpler than the compositional alternative.

Do not inherit solely for implementation reuse.

## Favor capabilities over taxonomies

When modeling behavior, ask what an object can do before building a class taxonomy around what it is.

Small interfaces or contracts are preferred when behavior must be interchangeable.

## Avoid deep hierarchies

Deep inheritance trees make behavior origin, overrides, invariants, and change impact harder to understand. Growing hierarchies should trigger a design review for possible composition.

## Do not over-engineer composition

Composition is a default, not an excuse for unnecessary layers.

If direct fields and functions provide a clear design, do not replace them with chains of providers, managers, services, repositories, and interfaces without a concrete need.

**Compose behavior by default. Use inheritance deliberately.**
