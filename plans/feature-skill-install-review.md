# Independent review — feature/skill-install

- **Verdict**: APPROVE (with minor suggestions, none blocking)
- **Reviewed commit**: 1606c849b629ca79f77b00d3fca865154c210f94 (`Add skill install command with embedded SKILL.md for coding agents`), parent ad8a5e5 (master tip). Note: the handoff's diff base `1606c849...` equals the branch tip, so that range is empty; the real delta is `master...feature/skill-install` (1 commit, 7 files, +202).
- **Gate**: `cargo test --offline` — **54 passed, 0 failed** (includes the 5 new tests in `tests/skill_command.rs`). `cargo clippy --offline --all-targets` — no new warnings from this change (remaining warnings are pre-existing test dead-code). `cargo fmt --check` — drift exists on master already (`--sort` help wrap in `src/cli.rs`); this branch adds none.

## Summary

Small, clean feature: embeds `skills/redmine/SKILL.md` via `include_str!`, adds `redmine skill install|show`, installs to `~/.agents/skills/redmine/SKILL.md`, and documents it in the README. The install path matches the Agent Skills convention, and I verified against pi's own skill-loader tests (present on this machine in pi session logs) that pi loads skills from `.agents/skills` directories and treats `~/.agents/skills` as the user-global dir — so the README's pi claim is accurate. Tests exercise the real compiled binary and assert rendered stdout plus on-disk state, not internals.

## Plan adherence (spec = handoff "Additional instructions"; no plan doc was provided)

| Focus area | Verdict | Notes |
|---|---|---|
| Correctness of install paths and HOME handling | matches | `directories::BaseDirs` (already a dep, no Cargo.toml change) → `~/.agents/skills/redmine/SKILL.md`. Tests isolate HOME per child process. Manual runs: success path prints `installed\t<path>` exit 0; unwritable target → exit 5. Empty `HOME` falls back to the passwd home via dirs-sys getpwuid (verified live) — defensible shell-like behavior, makes the `home_dir()` error branch nearly unreachable on Unix. |
| embed/include_str correctness | matches | `include_str!("../../skills/redmine/SKILL.md")` resolves from `src/commands/`; `skill show` output is byte-identical to the repo file (tested and re-verified manually). File is git-tracked so a future `cargo publish` would package it; no runtime dependency on the repo. |
| Frontmatter validity per agentskills.io | matches | `---` delimiters at file start; `name: redmine` (lowercase, hyphen-free, 7 chars ≤ 64); `description` 207 chars ≤ 1024, no characters that break plain YAML scalars; directory name matches `name`. Also satisfies pi's loader (name + description required — a skill missing description is skipped). |
| Test quality (rendered CLI output) | matches | All 5 tests spawn `CARGO_BIN_EXE_redmine` and assert actual stdout/stderr/exit codes and the installed file. See spot-checks below. |
| README accuracy | matches | New "Agent skill" section: install path, `skill show`, embed note, manual-install alternative all match behavior. `redmine --help` lists `skill`; subcommand help is accurate. The pi-discovery claim verified as above. |

## Bugs & risks

**BLOCKING**: none.

**MAJOR**: none.

**MINOR**

1. **SKILL.md exit-code list omits code 10** — `skills/redmine/SKILL.md:43`. README documents `10 = other HTTP`, the binary emits it (`src/main.rs` exit_code fallback), but the agent-facing skill lists 2,3,4,5,6,11–16 only. An agent reading the skill will misclassify a 10. Fix: add `10 other HTTP` to the list.
2. **Install silently overwrites an existing SKILL.md** — `src/commands/skill.rs:34`. Re-running install is idempotent by design (good), but a user-customized file is clobbered with no signal. Fix (cheap): have `install_into` report whether the file already existed and print `overwritten\t<path>` vs `installed\t<path>`, and say "overwrites any existing file" in the README/help text.
3. **Tests only redirect `HOME`, which does not control home resolution on Windows** — `tests/skill_command.rs:57,75,87`. `deploy.sh` explicitly ships a Windows binary; on Windows `directories` resolves the profile independently of `HOME`, so these tests would target the real user profile. Fix: `#[cfg(unix)]` on the three install tests, or also set `USERPROFILE` for the child.
4. **Install failures surface as `config error: io error: ...` (exit 5)** — `src/commands/skill.rs:31-34` via the blanket `From<io::Error>` in `src/error.rs`. Verified live: `HOME=/nonexistent redmine skill install` → `error: config error: io error: Permission denied`. Miscategorized label for a filesystem problem. Fix: wrap as `Error::Config(format!("cannot install skill to {}: {e}", path.display()))` (keeps exit 5, better message).

**NIT**

1. **Non-atomic write** — `src/commands/skill.rs:34`. A crash mid-`fs::write` leaves a truncated SKILL.md that agents will still discover. Write to a temp file in the same dir + rename for atomicity.
2. **Install test asserts only `stdout.contains("installed")`** — `tests/skill_command.rs:67`. A regression dropping the path from the output would pass. Assert the exact `installed\t<expected path>` line.
3. **Frontmatter test doesn't enforce spec limits or YAML validity** — `tests/skill_command.rs:28-50`. It checks presence of name/description but not name ≤ 64 / charset, description ≤ 1024, nor that the description parses as YAML (pi's loader skips the whole skill with `parse_failed` on invalid YAML). A future `: ` inside the description would silently disable discovery. Cheap hardening: assert name matches `^[a-z0-9-]+$`, len caps, and no `": "` in the description value.
4. **Pre-existing `cargo fmt` drift on master** (`src/cli.rs` `--sort` help wrapping). Not from this branch; fix separately so the next branch doesn't inherit the noise.
5. `redmine skill --help` shows irrelevant global flags (`--url`, `--key`, `--format`). Pre-existing CLI shape shared with `config`; not worth changing here.

## Test quality (spot-checked 3 of 5)

- `skill_show_matches_repo_file` — real: runs the built binary, byte-compares `skill show` stdout with the repo file. Directly guards the `include_str!` embed. Not trivially passing.
- `skill_install_writes_file_under_home` — real: child process with isolated `HOME`, asserts exit 0, exact on-disk content equality with the repo file, and rendered stdout. This is rendered-UI-style validation, not an internals test (only weakness: the loose stdout assertion, NIT 2).
- `skill_install_is_idempotent` — real double-submit check: two installs, then content equality. Covers the handoff's idempotency category.

Missing coverage (minor): no test for the no-HOME/unwritable-home error path (exit 5), and no test that `skill show` output ends with a newline (currently true).

## Suggested changes before merge (prioritized)

1. Add `10 other HTTP` to the SKILL.md exit-code list (factual inaccuracy in shipped agent-facing content).
2. Distinguish overwrite vs fresh install in output/README (data-loss-adjacent surprise).
3. Guard the HOME-based tests for non-Unix or set `USERPROFILE` too, given the Windows binary in `deploy.sh`.
4. Improve the install error message beyond the generic `config error: io error`.
5. (optional) Strengthen stdout/frontmatter assertions; atomic write.

None of these block merge on correctness grounds; the code is correct per the Agent Skills path/frontmatter conventions and pi's actual discovery behavior.
