# Kepler project rules

Kepler is a local Rust terminal tool for finding, learning, and explaining
commands. The MVP supports Fish and keeps command selection in a
ratatui interface. It inserts a selected command into the shell prompt. It
does not run the selected command.

Keep the codebase small. Document any rule exception in the relevant review.

## Technical choices

Rust, Ratatui, and Crossterm are already in the project. Clap, Rusqlite, and
Nucleo are planned dependencies; add them only with the code that uses them.

- Rust 2026 on the latest stable toolchain. The last verified toolchain is
  Rust 1.98.1.
- One Cargo package and one binary.
- ratatui 0.30.2 and crossterm 0.29.0, currently the latest stable releases
  verified for this project.
- clap 4.6.7 with its derive feature when the CLI needs it.
- rusqlite 0.40.2 with bundled SQLite for local storage.
- nucleo 0.5.0 for fuzzy matching and its worker-backed matching engine.
- Fish is the only supported shell in the MVP.
- The MVP works offline and has no model service, daemon, or async runtime.

Use the latest stable compatible release when adding or refreshing a dependency.
The versions above are the verified baseline on 2026-09-18, not a reason to
hold the project on old releases. Update the manifest and lockfile together,
review breaking changes, then run the full checks. Do not add a dependency for
a small function that the standard library can handle.

## Development setup

Use stable Rust with Cargo, rustfmt, and Clippy, plus Fish for shell checks.
Bundled SQLite also needs a C compiler and linker. Run commands from the
repository root. Keep `Cargo.lock` under version control for this application
when committing implementation changes. Dependency updates must be deliberate.

Local plans may be available in `docs/mvp.md`, `docs/foundations.md`, and
`docs/technical-plan.md`. That directory is intentionally ignored and may be
absent from another checkout. This file contains the required project rules.

## Repository layout

Keep the first implementation flat:

```text
src/
  main.rs       process setup and top-level error handling
  cli.rs        command-line arguments and modes
  app.rs        application state and event loop decisions
  ui.rs         ratatui layout and rendering
  search.rs     query normalization and fuzzy ranking
  store.rs      SQLite schema and parameterized reads and writes
  catalog.rs    small curated command catalog
shell/
  kepler.fish   Fish functions and prompt insertion hook
tests/
  fixtures/     small checked-in input samples
```

Create a module only when its code has a clear owner. Do not create `utils`,
`helpers`, `services`, `repositories`, `interfaces`, or empty placeholder
modules. Add a new directory only for a cohesive group that has a real caller.
Integration tests belong in `tests/`. Keep test fixtures in
`tests/fixtures/`, not in source code or user data directories.

## File and function limits

- `src/main.rs` is at most 100 physical lines.
- Every other handwritten Rust source file is at most 300 physical lines.
- `shell/kepler.fish` is at most 300 physical lines.
- Test source files are at most 300 physical lines.
- Counts include comments, blank lines, and inline tests.
- Documentation, fixture data, lockfiles, and generated files are exempt.
  Handwritten Rust catalog files still have the 300-line limit.
- Split a catalog only when it has become a meaningful separate catalog, not
  just to evade the limit.
- Aim for functions under 50 physical lines. A longer function needs a clear
  reason and should remain easy to read.

Keep meaningful logic within three hops. Imports and thin rendering calls do
not count as logic hops. If a feature requires tracing through more than
three layers, move the decision closer to the code that owns it.

## Rust structure and naming

Order each file consistently: imports, public types and constants, private
types, public functions, private helpers, then tests. Keep related types and
their methods together. Prefer a direct function over a trait with one
implementation. Keep modules private by default and use `pub(crate)` when a
cross-module call is necessary.

Use Rust conventions without exceptions for style:

- `snake_case` for files, modules, functions, fields, and local variables.
- `UpperCamelCase` for structs, enums, traits, and type aliases.
- `SCREAMING_SNAKE_CASE` for constants and static values.
- Use names that describe the command or data being handled. Avoid vague
  names such as `thing`, `data`, `manager`, and `misc`.
- Keep comments for non-obvious constraints, privacy boundaries, or terminal
  behavior. Do not narrate the code.

Use `Result` for failures the caller can handle. Keep error context close to
the operation that failed. Do not hide errors in broad fallbacks.

## Fish and terminal behavior

On cancel or failure, preserve the original Fish command line and cursor.
On selection, replace the whole line with the exact selected command and move
the cursor to its end. Restore the terminal on every exit path. Reserve stdout
for the selected text in selection mode; render the TUI and diagnostics through
stderr or the terminal. Never evaluate or automatically execute a selection.

Additive Fish hooks must preserve existing hooks. Do not overwrite a user's
binding or install a global binding without an explicit setup action. Private
Fish helpers use the `__kepler_` prefix. Public functions use `kepler_` names.

## Data and indexing rules

The MVP stores exact raw command text, execution count, and last-used time.
Do not parse a separate command name or collect project or sequence data yet.
Update counts atomically so simultaneous terminals do not overwrite each other.
Selecting a result does not count as execution.

Recording is opt-in. Pause, ignore, delete, and forget must work before real
history is retained. Skip private Fish sessions, leading-space commands,
internal recording calls, and unsupported multiline input. Do not collect
environment variables or command output. Exact commands can contain secrets;
do not promise perfect filtering. Never use real private history as test data.

MVP indexing combines bundled examples with commands observed by the Fish
execution hook. `PATH` inventory and `kepler learn <command>` help parsing are
later features. Never recursively run every executable with `--help`. Later
parsing must retain sources and mark unknown fields instead of inventing them.

The MVP ranking combines fuzzy query relevance with frequency and recency.
Use a simple stepped decay for older use, then tune it from observed behavior.
Do not add a background indexer, network search, or language model dependency.

SQL must stay in `store.rs`, use parameters, and have a small visible schema.
Do not log command text or database contents by default.

## Verification

After changing Rust code, run:

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Run `cargo build --release` when checking a release or packaging change. When
`shell/kepler.fish` exists, run `fish --no-execute shell/kepler.fish` and do an
interactive Fish smoke test for shell changes. Source inspection alone does
not prove prompt insertion, cursor preservation, or terminal restoration.

Docs-only changes do not need Cargo checks. Keep checks proportional to the
files changed, but never claim a runtime or interactive check without running
it.

## Git and change hygiene

Inspect `git status --short` before and after focused work. Preserve unrelated
user changes. Stage exact paths when staging is requested. Do not force ignored
planning files into Git. Do not commit, push, or create a pull request unless
the user asks. Remove files only when that is part of the requested change.

Keep changes narrow. Delete obsolete code instead of leaving commented-out
versions. Avoid speculative configuration, abstractions, compatibility layers,
and tests that only repeat the implementation. Every new file needs a clear
caller and every new dependency needs a concrete MVP use.
