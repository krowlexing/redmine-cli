# Review: feature/issue-parent-children

- **Verdict**: APPROVE
- **Reviewed commit**: d7aa703 (`feature/issue-parent-children`), diff vs master c658281 — 5 files, +151/−9
- **Gate**: `cargo test` 67 passed / 0 failed (16 test binaries + doc-tests); `cargo clippy` clean for lib/bin (test-binary dead-code warnings are a pre-existing pattern from the shared `tests/common` module); `cargo build` OK
- **Note on the diff range**: the handoff's range `d7aa703...feature/issue-parent-children` is empty because d7aa703 *is* the branch tip. The actual feature diff is `c658281..d7aa703` (commits 13e0b69, d7aa703). Reviewed that.

## Summary

Correct, small-scope implementation of parent/children display in `issue show`. The model matches the real Redmine API payload shapes exactly, the `include=children` request is right in both `--notes` variants, and the rendered output follows the existing block style. All tests pass; I verified end-to-end against a local HTTP mock that the rendered output and the wire query are as intended.

## Plan adherence (spec: parent task id + children block in issue show)

| Spec item | Status | Evidence |
|---|---|---|
| Parent task id in issue show | matches | `src/output.rs:203-206` — `parent\t<id>` in kv block, placed after `project` |
| Children block in issue show | matches | `src/output.rs:222-229` — blank line + `CHILDREN` header + `id\ttracker\tsubject` rows, before `DESCRIPTION` |
| `include=children` query change | matches | `src/commands/issue.rs:96-100` — both variants (`attachments,children` and `attachments,children,journals`) |
| Mock query recording (tests/common) | matches | `tests/common/mod.rs` — `query` field added to `RecordedRequest`; `parse_request` splits the request target on the first `?` |
| Rendered output quality | matches | Block style consistent with existing `DESCRIPTION`/`JOURNALS` blocks; tab-separated, no decoration (fits the project's agent-friendly format) |
| Test coverage | matches | 6 new tests: query variants ×2, parent placement, children block, absence case, JSON format |

## Model correctness vs Redmine API payload

- `parent`: Redmine emits `"parent": {"id": N}` in the issue payload (only `id`, present by default when the issue has a parent — no include needed). `ParentRef { id }` matches exactly.
- `children`: requires `include=children`; Redmine emits `[{id, tracker: {id, name}, subject}]`. `ChildIssue` matches exactly; `tracker`/`subject` have `#[serde(default)]` for defensiveness.
- Unknown extra fields are ignored (no `deny_unknown_fields`), so future Redmine additions won't break deserialization.
- `Issue` is shared with the list path; new fields are `Option` + default, so list responses (which never carry children) still deserialize.

## Wire behavior (verified with a local mock server)

- No `--notes`: `GET /issues/100.json?include=attachments%2Cchildren`
- With `--notes`: `GET /issues/100.json?include=attachments%2Cchildren%2Cjournals`
- reqwest percent-encodes the comma (`%2C`). Rack decodes percent-encoding before param parsing, so Redmine sees the comma — and this is identical to the pre-existing `attachments%2Cjournals` behavior, so no new risk.
- Rendered tab/pretty output confirmed: `parent\t99` directly after `project`; `CHILDREN` block with `101\tFeature\tFirst subtask` rows between the kv lines and `DESCRIPTION`. JSON contains `"parent":{"id":99}` and the children array.

## Bugs & risks

**BLOCKING**: none.

**MAJOR**: none.

**MINOR**:
1. **Weak query assertion in the no-notes test** — `tests/issue_show_parent_children.rs:49-53` (`show_requests_children_in_include`) asserts `children` and `attachments` are present but not that `journals` is *absent*. If `--notes` gating regressed to always-on, this test still passes. No test pinned the include set before this change either, so this is a pre-existing gap this branch could have closed. Fix: add `assert!(!req.query.contains("journals"), ...)` to the first test.
2. **Docs/skill not updated** — `skills/redmine/SKILL.md:24` and the README `issue show` example don't mention parent/children display. Agents that discover the CLI via the skill won't know hierarchy is surfaced. One line each would do.

**NIT**:
3. JSON format serializes `"parent":null` / `"children":null` when absent (no `skip_serializing_if`). Consistent with existing fields (`closed_on`, `journals`), so not a regression — noting only in case clean JSON output is ever wanted.
4. `parent` renders as a bare id with no subject — that is all the Redmine parent ref carries, so this matches the payload; showing a subject would cost a second request. Acceptable per spec.
5. `req.query.contains("children")` is a substring check on the raw (still percent-encoded) query; a param named e.g. `grandchildren` would false-match. Practically impossible here; exact match on the decoded include value would be tidier.

## Test quality (spot-checked 3)

- `show_prints_parent_id_after_project` — real behavioral depth: asserts the parent line's *index* is exactly one after the project line, i.e. placement, not just presence. Not trivially passing.
- `show_prints_children_block` — good depth: asserts the block header, both exact tab-separated rows, and a negative assertion (`!stdout.contains("parent\t")`). Exercises the real binary against the mock server (E2E, rendered output — not just the handler).
- `show_requests_children_in_include` — genuine wire-level assertion via the new `query` recording (the common-module change is exactly what enables it), but weaker than it could be (see MINOR 1).

## Suggested changes before merge

1. (minor) Add `assert!(!req.query.contains("journals"))` to `show_requests_children_in_include`.
2. (minor) Mention parent/children display in `skills/redmine/SKILL.md` and the README `issue show` example.

Neither blocks merge. Reviewed read-only; no source files modified.
