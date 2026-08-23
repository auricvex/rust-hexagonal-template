# AGENTS.md

Binding rules for every AI agent — and human — writing code in this project.
Read this file before your first change and re-check it before you declare
work done. Everything here is enforced by the toolchain where possible; the
rest is enforced by review.

This is proprietary software (`LicenseRef-Proprietary`, publishing disabled).
Never change `license` or `publish` in any `Cargo.toml`.

---

## Non-negotiables (TL;DR)

1. **YAGNI + KISS.** Build only what the current task needs, the simplest way
   that works. See §1.
2. **Files stay ≤ 350 code lines** (comments and doc comments excluded).
   Split larger files along idiomatic Rust seams. See §2.
3. **Formatting and lints are law.** `rustfmt.toml` (nightly rustfmt),
   `clippy.toml` (MSRV), and `[workspace.lints]` in the root `Cargo.toml`
   define the policy; `just ci` is the gate. See §3.
4. **No panics, unwraps, expects, unchecked indexing, debug prints, or
   `todo!()`/`unimplemented!()` in production code.** Return `Result`/`Option`
   and model errors as types. See §4.
5. **Everything is documented:** `//!` for crate/module docs, `///` on every
   public item (including fields and variants), `//` only for logic that is
   genuinely hard to judge from the code itself. See §5.
6. **Hexagonal architecture is mandatory.** Dependencies point inward;
   `domain` depends on nothing; adapters implement ports; `bins/*` is the only
   place that wires things together. See §6.
7. **Names encode the architecture.** Every type carries its role's suffix —
   `UserEntity`, `MoneyVo`, `UserRepository`, `RegisterUserUseCase`,
   `UserDto` — so the layer is visible from the name alone. See §7.
8. **All dependencies live in the root `Cargo.toml`**
   (`[workspace.dependencies]`); member crates inherit via
   `name.workspace = true`. No per-crate versions, ever. See §8.

---

## 1. Principles: YAGNI and KISS

These override personal taste, templates, and habit.

- Implement exactly what the task requires — no speculative configuration
  options, generic parameters, trait indirection, abstraction layers, or
  "we'll need it later" hooks. If a requirement doesn't ask for it, don't
  build it.
- Prefer boring, obvious Rust: plain structs and enums, functions, `Result`,
  constructor injection. Reach for macros, builder patterns, or async
  machinery only when the problem demands it.
- Keep the public API surface minimal. Items are private by default; `pub`
  only when another module or crate genuinely needs it (`unreachable_pub` is
  a deny-level lint — a stray `pub` fails the build).
- Delete dead code; never park stubs behind `todo!()`/`unimplemented!()`
  (both are denied) or commented-out blocks. If it isn't needed, remove it.
- When two designs both work, choose the one that is easier to read, delete,
  and test — not the more extensible one.
- If you believe a rule here conflicts with a requirement, stop and ask the
  maintainer instead of silently deviating.

## 2. File size: ≤ 350 code lines per `.rs` file

Hard limit. "Code lines" = non-blank lines that are not `//`, `///`, or `//!`
comments. Measure with:

```bash
# Single file
grep -cvE '^[[:space:]]*(//|$)' src/lib.rs

# Whole workspace — prints offenders (silence means all files pass)
grep -rcvE '^[[:space:]]*(//|$)' --include='*.rs' crates bins | awk -F: '$2 > 350 {print}'
```

(The codebase avoids `/* */` block comments; the check exempts only
`//`-style comments. Don't convert code into comments to dodge the limit.)

When a file approaches the limit, split it along these idiomatic seams:

- **Extract child modules** by responsibility, using the directory style this
  repo already uses (`dtos/mod.rs`, `use_cases/mod.rs`). Re-export the public
  surface from the parent module so import paths stay stable.
- **Give large types their own files** — an enum with many variants, a big
  error type, or a multi-trait struct each deserve their own module.
- **Move inline unit tests out:** declare `#[cfg(test)] mod tests;` backed by
  a `tests.rs` sibling, or use integration tests under `tests/`. Test code
  still counts toward the limit wherever it lives.
- **Group `impl` blocks by concern** in separate files, each owning one
  cohesive aspect of the type.

Do not game the metric: splitting unrelated code into fragments just to get
under 350 lines violates the cohesion the limit exists to protect. Comments
and doc comments are exempt because good documentation is never punished.

## 3. Toolchain compliance: rustfmt, clippy, MSRV

- **Format only with nightly rustfmt:** `just fmt` (apply) and
  `just fmt-check` (verify). `rustfmt.toml` enables unstable options that
  stable rustfmt silently ignores, so editor format-on-save produces wrong
  output unless it runs `cargo +nightly fmt` with `edition = "2024"` /
  `style_edition = "2024"` honored.
- **Clippy policy lives in three places**, all authoritative:
  - `Cargo.toml → [workspace.lints]` — lint *levels*: `pedantic` and
    `clippy::all` denied, selected restriction lints denied, `unsafe_code`
    forbidden, `missing_docs` denied.
  - `clippy.toml` — lint *parameters* and the MSRV (currently **1.98**,
    tracking current stable). Never use APIs newer than the MSRV; clippy
    already suppresses such suggestions.
  - Per-crate `[lints] workspace = true` — every member must inherit; never
    redefine lint levels locally.
- **Never weaken policy wholesale.** No new crate-level `#![allow(...)]`
  blankets. If one specific site truly needs an exception, use the narrowest
  possible `#[allow(...)]` directly on that item with a comment explaining
  why. If several sites hit the same lint, propose adjusting the workspace
  config (with rationale, like the existing entries) instead of sprinkling
  attributes.
- **`just ci` is the gate** (`fmt-check`, `clippy`, `test`, `doc`). Run it
  before declaring work finished; a red gate means the work is not done.
  `just fix` auto-applies safe fixes (clippy suggestions, then formatting).

## 4. Failure handling: no panics, ever

`unsafe_code` is forbidden outright, and these are **deny-level** lints in
production code — the build fails on any of them:

| Banned                              | Use instead                                                        |
| ----------------------------------- | ------------------------------------------------------------------ |
| `.unwrap()` / `.expect()`           | `?`, `match`, `ok_or(...)`, `unwrap_or_else(...)` with real handling |
| `panic!`, `assert!` (non-test)      | Return `Result` with a typed error                                 |
| `v[i]`, slice ranges                | `.get(i)`, iterators, or explicit bounds handling                  |
| `println!` / `eprintln!` / `dbg!`   | `tracing` (structured logging)                                     |
| `todo!()` / `unimplemented!()`      | Finish the implementation before committing                        |

Guidance:

- Model failures as types: domain/application errors live in
  `domain::errors` (and application-level errors alongside use cases);
  adapters map their technology's errors into those types at the boundary —
  raw driver/ORM/HTTP error types must not leak inward.
- Handle `None`/`Err` explicitly and meaningfully. Swallowing an error into
  a default value is not handling it; log-or-propagate deliberately.
- The `allow-*-in-tests` keys in `clippy.toml` make assertions and unwraps
  acceptable **in test code only**. Production paths are never exempt.

## 5. Documentation

`missing_docs = "deny"` makes documentation a compilation requirement, and
`RUSTDOCFLAGS="-D warnings"` makes any rustdoc warning (broken intra-doc
links included) fail `just doc`. Treat docs as part of the code:

- **Crate/module level — `//!`:** every `lib.rs` opens with a module doc
  stating the layer's role and its dependency constraints (see the existing
  crates for the expected style). Significant standalone modules get one too.
- **Public items — `///`:** every public module, struct, enum, trait,
  function, const — including public fields and enum variants. Document
  semantics, invariants, units, and which error variants a function can
  return. A doc that merely restates the item's name is a bug, not docs.
- **Inline comments — `//`:** only where the code is genuinely hard to judge
  from itself, and explain *why*, never *what*. Narrating obvious code is
  noise.
- Use intra-doc links (`[`DomainError`]`) so docs stay navigable; broken
  links fail the doc gate. Keep docs updated in the same change as the code —
  stale docs are defects.

## 6. Architecture: hexagonal (ports and adapters)

Current layout (new code follows the same placement):

```text
        ┌──────────────────────────────────────────────────┐
        │ bins/*  — composition root                       │
        │ the ONLY place that constructs and wires adapters│
        └────────┬─────────────────────────────┬───────────┘
                 ▼                             ▼
 ┌───────────────────────────────┐ ┌────────────────────────────────┐
 │ infrastructure/incoming/*     │ │ infrastructure/outgoing/*      │
 │ driving adapters (http-axum): │ │ driven adapters (seaorm-       │
 │ transport ⇄ DTO ⇄ use case    │ │ postgres): implement ports ⇄ DB│
 └───────────────┬───────────────┘ └───────────────┬────────────────┘
                 ▼                                 ▼
        ┌──────────────────────────────────────────────────┐
        │ crates/application — use cases + DTOs            │
        └───────────────────────────┬──────────────────────┘
                                    ▼
        ┌──────────────────────────────────────────────────┐
        │ crates/domain — entities, value_objects,         │
        │ services, ports/repositories (traits), errors    │
        │ depends on NOTHING                               │
        └──────────────────────────────────────────────────┘

 crates/shared/* (migrations): cross-cutting support, no business logic
```

The dependency rule: **all arrows point inward**. The inside never mentions
the outside.

1. `domain` depends on nothing — no workspace crates, no frameworks. Keep its
   external dependencies near zero and side-effect free. Its `Cargo.toml`
   `[dependencies]` staying empty of infrastructure is a design invariant.
2. `application` depends only on `domain` (plus pure support crates if truly
   unavoidable). Never on infrastructure.
3. Adapters depend on `application`/`domain` — never on each other. An
   incoming adapter must not call an outgoing adapter directly; it goes
   through use cases and ports.
4. **Ports are owned by the inside.** Traits for persistence live in
   `domain/repositories` (all other outward interactions in `domain/ports`
   — see *Ports vs. repositories* below); the domain states what it
   *needs*, adapters supply what it asked for.
5. **Boundaries are translated.** Transport types (Axum request/response
   bodies) and persistence types (SeaORM entities) stop at their adapter.
   Crossing into the application happens through DTOs
   (`application/dtos`) and port interfaces. Adapter type names appearing
   inside `domain`/`application` is always a bug.
6. `bins/*` contains only bootstrap and wiring: construct outgoing adapters,
   inject them into use cases as port implementations, hand use cases to
   incoming adapters, start the runtime. No business logic in binaries.
7. `crates/shared/*` holds cross-cutting mechanics only — no business rules.

### Ports vs. repositories: which folder gets a trait

Classic hexagonal architecture needs only "ports" — a repository *is* a
driven port. This workspace deliberately keeps **both** folders as a topical
split: persistence contracts for the domain's own aggregates live apart from
every other outward capability, so `ports/` never becomes a dumping ground.
The split is convention, not mechanism: both kinds are owned by the domain,
implemented by outgoing adapters, and injected at the composition root in
exactly the same way.

Deciding where a new trait goes:

| The contract…                                                     | Belongs to                    | Named                |
| ----------------------------------------------------------------- | ----------------------------- | -------------------- |
| persists or retrieves **this domain's aggregates** (`UserEntity` & co.) | `domain/repositories`   | `<Concept>Repository` |
| is any **other outward capability** — send mail, publish events, hash passwords, read the clock, store files, call an external API | `domain/ports` | `<Capability>Port` |

Litmus tests:

- A repository speaks **collection language about entities**: methods like
  `find_by_id`, `save`, `delete`, `exists`, and every signature traffics in
  domain types (`…Entity`, `…Vo`) only. If a method returns or accepts DTOs
  or transport types, it is not a repository.
- A port describes an **action or capability**, usually verb-shaped
  (`send`, `publish`, `hash`, `now`), and its signatures serve that
  capability, not aggregate storage.
- Still unsure? Repositories answer *"where do aggregates sleep?"* —
  everything else is a port.

Never invent a third home for contracts (`gateways/`, `providers/`,
`services/` in an adapter, …), never merge the two folders, and never move
one side's traits into the other to "simplify".

Where new work goes:

| Task                                        | Location                                            |
| ------------------------------------------- | --------------------------------------------------- |
| Business concept / invariant                | `domain/entities`, `domain/value_objects`           |
| Logic spanning entities                     | `domain/services`                                   |
| "Persist/fetch X" contract                  | trait in `domain/repositories`                      |
| Other outward contract (mail, queue, …)     | trait in `domain/ports`                             |
| Implementation of a port                    | new/existing crate under `infrastructure/outgoing/` |
| New user-facing capability                  | use case in `application/use_cases` + DTO           |
| Endpoint                                    | `infrastructure/incoming/http-axum`                 |
| Schema change                               | new migration in `shared/migrations` (never edit an applied migration) |
| New external technology                     | new crate under the matching `infrastructure/` side |
| Container images / dev stack for `bins/x`   | `docker/x/` — created only if needed (§9)           |

## 7. Naming conventions: names encode the architecture

Every workspace-authored type whose job is one of the architectural roles
from §6 carries that role's suffix, so the name alone tells you which layer —
and which side of the hexagon — it lives on. The suffix is not decoration:
it is how role boundaries stay visible in review, diffs, and grep.

| Role (where it lives)                         | Name pattern                        | Examples                                    |
| --------------------------------------------- | ----------------------------------- | ------------------------------------------- |
| Entity (`domain/entities`)                    | `<Concept>Entity`                   | `UserEntity`, `OfficeEntity`                |
| Value object (`domain/value_objects`)         | `<Concept>Vo`                       | `MoneyVo`, `EmailVo`                        |
| Domain service (`domain/services`)            | `<Concept>Service`                  | `PricingService`                            |
| Repository trait (`domain/repositories`)      | `<Concept>Repository`               | `UserRepository`                            |
| Other driven port trait (`domain/ports`)      | `<Capability>Port`                  | `MailSenderPort`, `EventPublisherPort`      |
| Use case (`application/use_cases`)            | `<VerbPhrase>UseCase`               | `RegisterUserUseCase`                       |
| DTO (`application/dtos`)                      | `<Concept>Dto`                      | `UserDto`, `RegisterUserDto`                |
| Transport payload (`infrastructure/incoming`) | `<Action>Request` / `<Concept>Response` | `CreateUserRequest`, `UserResponse`     |
| Error type                                    | `<Scope>Error`                      | `DomainError`, `UserRepositoryError`        |
| Adapter implementation of a port              | `<Tech><Concept>`                   | `SeaOrmUserRepository`                      |
| Persistence row (SeaORM, adapter-internal)    | framework's `Model` / `ActiveModel`; alias `<Concept>Row` if a local alias helps | `user::Model` → `UserRow` |
| Migration (`shared/migrations`)               | `m<YYYYMMDD>_<HHMMSS>_<snake_description>` | `m20260822_120000_add_orders_table`  |
| Binary / package                              | lowercase, no suffix                | `example`, `http-axum`                      |

Rules:

- **Suffixes apply to types the workspace authors.** Framework-owned types
  keep their framework's names (axum's `Router`, SeaORM's generated
  `Model`/`Entity`) — never rename or wrap them just to satisfy a suffix.
  In particular, SeaORM's per-table `Entity` is *not* a domain entity; it
  never leaves its adapter, and the domain's `<Concept>Entity` types are
  unrelated to it.
- **One stem per concept across layers.** The `User` concept is
  `UserEntity` in the domain, `UserRepository` as its port, `UserDto` at the
  application boundary, `CreateUserRequest`/`UserResponse` at the HTTP
  boundary, and `user::Model` inside the adapter. The stem never changes;
  only the suffix does.
- **A suffix-less concept type is a smell.** `struct User` should not exist:
  if it is an entity it is `UserEntity`; if it plays any other role it takes
  that role's suffix. Helper types that play no architectural role (builders,
  internal plumbing) are exempt and follow plain Rust conventions.
- **Banned patterns:** no `I`-prefixed interfaces (`IUserRepository`), no
  `...Trait`, no `...Impl`, no `Abstract...`/`...Base`. Adapter
  implementations are named by technology + concept
  (`SeaOrmUserRepository`), never `...Impl`.
- **Files follow their primary type:** a snake_case file named after the
  concept holds one primary type and is re-exported from the role's
  `mod.rs` — `UserEntity` in `entities/user.rs`, `MoneyVo` in
  `value_objects/money.rs`, `UserRepository` in `repositories/user.rs`,
  `RegisterUserUseCase` in `use_cases/register_user.rs`. This pairs with the
  splitting rules in §2.
- **Otherwise follow the Rust API guidelines:** `UpperCamelCase` types with
  acronyms capitalized (`HttpServer`), `snake_case` modules and functions,
  `SCREAMING_SNAKE_CASE` consts, getters without a `get_` prefix.

## 8. Dependency management: one source of truth

- **Every dependency is declared once**, in the root `Cargo.toml`
  `[workspace.dependencies]` — external crates *and* internal path crates.
  Member crates opt in with `name.workspace = true` in `[dependencies]`.
  Pinning a version inside a member crate is forbidden.
- **Features are configured centrally** on the workspace entry (see
  `sea-orm`'s features) so feature unification stays predictable. Don't
  re-enable or drop features per crate without changing the central entry.
- **Member manifests stay uniform:** inherit `version`, `edition`,
  `license`, `publish` from the workspace, and keep
  `[lints] workspace = true`. New crates must be added to
  `[workspace.members]` and, once referenced elsewhere, to
  `[workspace.dependencies]`.
- **Every new dependency needs a justification** (YAGNI applies to deps
  too): prefer `std` first, then existing workspace deps. Vet candidates for
  maintenance health, license compatibility with a proprietary product
  (MIT/Apache-2.0/BSD yes; GPL/AGPL no), and advisories via `just audit`.
- **Commit `Cargo.lock` together with manifest changes.** CI resolves with
  `--locked`, so a stale lockfile fails the build loudly.
- **Tool versions track their library's major:** e.g. `sea-orm-cli` stays
  `^2.0` while the workspace uses `sea-orm ^2.0` (`just setup` enforces
  this). Keep any such pairing aligned when bumping majors.

---

## 9. Containers: `docker/` mirrors `bins/`

Container assets live under `docker/` with a **folder-per-binary mapping** to
`bins/`: `bins/example` → `docker/example/`. The mapping is *may*, not *must*
— a binary gets a folder only if it actually needs images. Both directions of
the invariant hold:

- Never create or reference a `docker/<name>/` whose binary does not exist in
  `bins/`; an orphaned folder is a bug.
- When a binary needs containers, create its folder by copying an existing
  one and renaming **every** occurrence of the old name — the package/binary
  name, `cargo build --package`, the compose project `name:`, service names,
  database credentials, volume keys, and `DATABASE_URL`. The existing files
  carry comments marking each spot that must change.

Each `docker/<name>/` folder contains:

| File              | Purpose                                                                |
| ----------------- | ---------------------------------------------------------------------- |
| `Dockerfile`      | Multi-stage. Target `dev`: pinned rust toolchain + cargo-watch, source bind-mounted by compose (nothing baked in). Target `release` (the default): production image — cargo-chef dependency caching, `cargo build --release --locked`, minimal non-root runtime on `debian:*-slim`. |
| `dev.compose.yml` | Dev-only stack: the app service (builds `target: dev`, repository bind-mounted at `/app`, named volumes shadowing `target/` and the cargo registry) plus the outgoing services it needs (PostgreSQL today), wired with healthchecks and dev-only placeholder credentials. |

Standing rules:

- **The build context is always the repo root**
  (`docker build -f docker/<name>/Dockerfile .`; compose sets
  `context: ../..`). Paths inside these files assume it.
- **Keep the sync points aligned** whenever either side changes:
  - `EXPOSE` / compose port mapping ↔ the incoming adapter's bind port
    (`8080` in the template).
  - `ARG RUST_VERSION` ↔ the MSRV in `clippy.toml` — the build image tracks
    it; bump both together.
  - compose `DATABASE_URL` ↔ the postgres service's environment block.
- **Pin everything**: base images by specific tag, `cargo install --locked`,
  `cargo build --locked`. The reproducibility rules from §8 apply verbatim.
- Only dev-only placeholder credentials may appear in `dev.compose.yml`;
  real secrets never enter any file under `docker/` or anywhere else in the
  repo.

---

## Definition of done

Before you report a task complete, all of the following are true:

- [ ] `just ci` passes end-to-end (fmt-check, clippy, test, doc).
- [ ] No `.rs` file exceeds 350 code lines (§2 check command is silent).
- [ ] Architecture boundaries hold: inward crates know nothing of adapters;
      no transport or ORM types leaked past their boundary.
- [ ] Any new dependency went through §8 (centralized, justified, audited,
      lockfile updated).
- [ ] All public items documented; module docs reflect reality; no stale docs.
- [ ] New behavior has tests covering success *and* error paths; failure
      handling follows §4.
- [ ] Nothing left half-done: no `todo!()`, no `TODO:` comments, no dead
      code, no commented-out code.
- [ ] Changes touching a binary kept its `docker/<name>/` folder in sync
      (§9): names, ports, MSRV pin, credentials — and no orphaned folders.
- [ ] New types carry their §7 role suffix; no bare concept structs, no
      `I`-prefixed, `...Impl`, or `...Trait` names.
