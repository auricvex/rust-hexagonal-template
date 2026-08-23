# Justfile for blackstore — the single entry point for building, checking,
# linting, testing, formatting, and documenting. See https://github.com/casey/just.
#
# Policy notes (see Cargo.toml `[workspace.lints]`, rustfmt.toml, clippy.toml):
# - Formatting uses nightly rustfmt: rustfmt.toml enables unstable options
#   that stable rustfmt silently ignores, so the *full* policy only applies
#   via `cargo +nightly fmt`. `just fmt-check` is the CI/pre-commit gate.
#   Recipes below fail early with an install hint if nightly is missing.
# - Clippy levels live in the workspace lints; deny-level lints fail the
#   build with no extra flags, so plain `cargo clippy` is already a gate.
# - `missing_docs = "deny"` makes `cargo doc` a gate too: undocumented
#   public items fail the doc build, and RUSTDOCFLAGS="-D warnings" turns
#   every other rustdoc warning (e.g. broken intra-doc links) into an error.
#
# Quick start: `just setup` once per machine, bare `just` lists recipes,
# `just ci` runs the full gate.

# Toolchain used for formatting (nightly-only rustfmt options).
nightly := "nightly"

# In CI, require Cargo.lock to be up to date so resolution changes fail
# loudly instead of being silently committed by the build.
locked := if env_var_or_default("CI", "") != "" { "--locked" } else { "" }

# Shorthand aliases for the hot recipes.
alias c  := check
alias f  := fmt
alias t  := test
alias cb := clippy

# List every public recipe.
default:
    @just --list

# ── Setup ────────────────────────────────────────────────────────────────────

# One-time machine setup; optional extras: just setup all|nextest|audit|mutants.
setup *tools:
    #!/usr/bin/env bash
    set -eo pipefail

    tools=({{tools}})
    case " ${tools[*]} " in
        *" all "*) tools=(nextest audit mutants) ;;
    esac

    echo "==> checking rustup"
    if ! command -v rustup >/dev/null 2>&1; then
        echo "error: rustup is required but not installed. install it from https://rustup.rs, e.g.:" >&2
        echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
        exit 1
    fi

    echo "==> checking active rustc against MSRV 1.98 (clippy.toml)"
    version="$(rustc --version | awk '{print $2}')"
    major="${version%%.*}"
    minor="${version#*.}"
    minor="${minor%%.*}"
    if [ "$major" -lt 1 ] || { [ "$major" -eq 1 ] && [ "$minor" -lt 98 ]; }; then
        echo "error: active rustc $version is below MSRV 1.98 (pinned to current stable)." >&2
        echo "    fix it with: rustup update" >&2
        exit 1
    fi
    echo "    rustc $version OK"

    if rustup run {{nightly}} cargo fmt --version >/dev/null 2>&1; then
        echo "==> {{nightly}} toolchain with rustfmt already present (updates: rustup update {{nightly}})"
    else
        echo "==> installing the {{nightly}} toolchain with rustfmt (minimal profile)"
        rustup toolchain install {{nightly}} --profile minimal --component rustfmt
    fi

    echo "==> ensuring clippy is installed for the default toolchain"
    rustup component add clippy

    # sea-orm-cli generates the entities in crates/infrastructure/outgoing/
    # seaorm-postgres; keep its major version aligned with the workspace's
    # sea-orm dependency (^2.0).
    echo "==> checking sea-orm-cli against ^2.0 (matches the workspace sea-orm)"
    cli_version="$(sea-orm-cli --version 2>/dev/null | grep -oE '[0-9]+(\.[0-9]+)*' | head -n1 || true)"
    cli_major=0
    [ -n "$cli_version" ] && cli_major="${cli_version%%.*}"
    if [ "$cli_major" -ge 2 ]; then
        echo "    sea-orm-cli $cli_version OK"
    elif [ -n "$cli_version" ]; then
        echo "==> upgrading sea-orm-cli $cli_version to ^2.0 (compiles from source; may take several minutes)"
        cargo install sea-orm-cli@^2.0 --locked
    else
        echo "==> installing sea-orm-cli ^2.0 (compiles from source; may take several minutes)"
        cargo install sea-orm-cli@^2.0 --locked
    fi

    for tool in "${tools[@]}"; do
        case "$tool" in
            nextest) crate=cargo-nextest ;;
            audit)   crate=cargo-audit ;;
            mutants) crate=cargo-mutants ;;
            cargo-*) crate="$tool" ;;
            *)
                echo "error: unknown optional tool '$tool' (known: nextest, audit, mutants, all)" >&2
                exit 1
                ;;
        esac
        if command -v "$crate" >/dev/null 2>&1; then
            echo "==> $crate already installed"
        else
            echo "==> installing $crate (compiles from source; may take several minutes)"
            cargo install "$crate" --locked
        fi
    done

    echo
    echo "setup complete. try:"
    echo "    just        # list recipes"
    echo "    just ci     # full verification gate"
    echo "    just info   # toolchain versions"

# ── Guards ───────────────────────────────────────────────────────────────────

# Fail early with guidance when the nightly toolchain lacks rustfmt.
_require-nightly-rustfmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! rustup run {{nightly}} cargo fmt --version >/dev/null 2>&1; then
        echo "error: the '{{nightly}}' toolchain (with rustfmt) is required but not usable." >&2
        echo "       fix it with: just setup" >&2
        exit 1
    fi

# ── Formatting ───────────────────────────────────────────────────────────────

# Apply nightly rustfmt to the whole workspace.
fmt: _require-nightly-rustfmt
    cargo +{{nightly}} fmt --all

# Verify formatting without modifying files (the CI / pre-commit gate).
fmt-check: _require-nightly-rustfmt
    cargo +{{nightly}} fmt --all --check

# ── Linting ──────────────────────────────────────────────────────────────────

# Run clippy on all targets and features. Deny-level lints fail the build.
clippy:
    cargo clippy --workspace --all-targets --all-features {{locked}}

# Auto-fix clippy findings, then re-run the gate to surface what's left.
clippy-fix:
    cargo clippy --workspace --all-targets --all-features --fix --allow-staged --allow-dirty
    cargo clippy --workspace --all-targets --all-features

# ── Checking & building ──────────────────────────────────────────────────────

# Fast type-check of the whole workspace (no codegen, no lints).
check:
    cargo check --workspace --all-targets --all-features

# Build the whole workspace (extra args pass through, e.g. --release).
build *args:
    cargo build --workspace --all-features {{locked}} {{args}}

# ── Testing ──────────────────────────────────────────────────────────────────

# Run the full workspace test suite with all features.
test *args:
    cargo test --workspace --all-features {{locked}} {{args}}

# Run tests via cargo-nextest — faster and better isolated than cargo test.
nextest:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-nextest >/dev/null 2>&1; then
        echo "error: cargo-nextest is not installed. install it with:" >&2
        echo "           just setup nextest" >&2
        exit 1
    fi
    exec cargo nextest run --workspace --all-features {{locked}}

# ── Documentation ────────────────────────────────────────────────────────────

# Build docs; fails on undocumented public items and any rustdoc warning.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features {{locked}}

# Build docs and open them in your browser.
doc-open:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --open

# ── Supply chain & mutation testing ──────────────────────────────────────────

# Audit dependencies against the RustSec advisory DB (cargo-audit).
audit:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-audit >/dev/null 2>&1; then
        echo "error: cargo-audit is not installed. install it with:" >&2
        echo "           just setup audit" >&2
        exit 1
    fi
    exec cargo audit

# Mutation-test the workspace with cargo-mutants (args pass through).
mutants *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-mutants >/dev/null 2>&1; then
        echo "error: cargo-mutants is not installed. install it with:" >&2
        echo "           just setup mutants" >&2
        exit 1
    fi
    exec cargo mutants --workspace --all-features {{args}}

# ── Aggregates ───────────────────────────────────────────────────────────────

# Full verification gate: what CI would run. Everything must pass.
ci: fmt-check clippy test doc

# Auto-fix everything fixable: clippy suggestions, then formatting last.
fix: clippy-fix fmt

# ── Maintenance ──────────────────────────────────────────────────────────────

# Remove build artifacts and stray rustc ICE dumps.
clean:
    cargo clean
    @rm -f rustc-ice-*.txt

# ── Diagnostics ──────────────────────────────────────────────────────────────

# Show expected toolchain versions and the nightly fmt policy status.
info:
    @echo "just:     $(just --version)"
    @echo "cargo:    $(cargo --version)"
    @echo "rustc:    $(rustc --version)"
    @echo "clippy:   $(cargo clippy --version)"
    @echo "rustfmt:  $(cargo fmt --version)"
    @if command -v sea-orm-cli >/dev/null 2>&1; then \
        echo "sea-orm:  $(sea-orm-cli --version)"; \
    else \
        echo "sea-orm:  not installed (run: just setup)"; \
    fi
    @echo
    @echo "rustup toolchains:"
    @rustup toolchain list
    @echo
    @if rustup run {{nightly}} cargo fmt --version >/dev/null 2>&1; then \
        echo "nightly fmt policy: OK ('{{nightly}}' toolchain has rustfmt)"; \
    else \
        echo "nightly fmt policy: MISSING — run: rustup toolchain install {{nightly}} --component rustfmt"; \
    fi
    @echo
    @echo "MSRV is 1.98 (clippy.toml); the full fmt policy requires '{{nightly}}' rustfmt."
