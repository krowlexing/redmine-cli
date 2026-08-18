# feature/attachment-support — independent review

## Verdict
REQUEST CHANGES

## Reviewed commit
5c97393d0c22e3eabf0de5927fa9a0153e41ae52 (tip of feature/attachment-support)

## Gate status
- **Tests**: 53 passed, 0 failed
- **Typecheck**: clippy error (BLOCKING) - see bugs section

## Summary
The feature adds attachment listing and download capabilities to the Redmine CLI, including data models, output formatting, and integration with issue show. Implementation is functionally correct and well-tested, but contains a blocking clippy error, a logic bug in the show command's query parameters, and incomplete error handling during file downloads.

## Plan adherence
No plan was provided, so inferred requirements from implementation:

| Requirement | Status | Notes |
|-------------|--------|-------|
| List attachments on an issue | ✓ MATCHES | `issue attachments list <id>` works correctly |
| Download attachments by ID | ✓ MATCHES | `issue attachments download <id> [--output <path>]` works |
| Show attachment count in issue detail | ✓ MATCHES | Footer shown when attachments present |
| Attachment data models | ✓ MATCHES | Attachment struct with appropriate fields |
| Output formatting (tab/pretty/json) | ✓ MATCHES | Consistent with existing output modes |
| Error handling | △ PARTIAL | HTTP errors handled, but file write errors leave partial files |

## Bugs & risks

### BLOCKING

**1. Clippy error prevents clean compilation**
- **File**: `src/commands/issue.rs:63`
- **Issue**: `q.push(("offset", &""));` creates unnecessary reference
- **Fix**: Change to `q.push(("offset", ""));`
- **Impact**: Fails `cargo clippy -- -D warnings`

**2. Mutually exclusive query parameters in show()**
- **File**: `src/commands/issue.rs:96-100`
- **Issue**: Query parameter is EITHER `include=journals` OR `include=attachments`, never both
- **Root cause**: Conditional logic chooses one based on `include_journals` flag
- **Current behavior**: When using `--notes`, attachments are not requested. When not using `--notes`, journals are not requested.
- **Expected behavior**: Should request both when available (attachments always, journals when `--notes` is used)
- **Fix**: Change query construction to always include attachments, conditionally include journals:
  ```rust
  let mut query = vec![("include", "attachments")];
  if include_journals {
      // Redmine accepts comma-separated values
      query[0] = ("include", "attachments,journals");
  }
  let wrapper: IssueWrapper = client.get(&path, &query)?;
  ```
- **Impact**: `issue show --notes` does not show attachment footer because attachments are not fetched

### MAJOR

**3. Partial file left behind on write error**
- **File**: `src/client.rs:128-135`
- **Issue**: If `resp.bytes()` succeeds but `std::fs::write()` fails, no file cleanup occurs
- **Scenario**: Disk full, permission denied, directory missing after bytes fetched
- **Current code**:
  ```rust
  let bytes = resp.bytes().map_err(|e| Error::Network(e.to_string()))?;
  std::fs::write(output_path, bytes).map_err(|e| Error::Network(format!("failed to write file: {}", e)))?;
  ```
- **Fix**: Use atomic write pattern (write to temp file, then rename):
  ```rust
  let temp_path = output_path.with_extension("tmp");
  std::fs::write(&temp_path, bytes).map_err(|e| Error::Network(format!("failed to write file: {}", e)))?;
  std::fs::rename(&temp_path, output_path).map_err(|e| Error::Network(format!("failed to finalize file: {}", e)))?;
  ```
- **Impact**: User gets corrupted/partial file with no indication of failure state

**4. Missing newline at end of file**
- **File**: `src/commands/issue.rs`
- **Issue**: File ends without newline after `attachments()` function
- **Fix**: Add newline at end of file
- **Impact**: Violates POSIX text file conventions, may cause issues with some tools

### MINOR

**5. Unhelpful default output filename**
- **File**: `src/commands/issue.rs:144-146`
- **Issue**: When `--output` is not provided, defaults to attachment ID (e.g., "10") as filename
- **Current behavior**:
  ```rust
  let output_path = output.unwrap_or_else(|| {
      std::path::PathBuf::from(format!("{}", id))
  });
  ```
- **Expected behavior**: Should use attachment's actual filename from metadata (requires fetching it first)
- **Fix**: Either require `--output` or fetch attachment metadata to get real filename
- **Impact**: Users get numeric filenames without extension, making files hard to identify

**6. No path validation for security**
- **File**: `src/commands/issue.rs:144-146`
- **Issue**: No validation that output path is within allowed directories
- **Risk**: Could potentially write to arbitrary locations if called from untrusted context
- **Fix**: Validate path doesn't contain `..` or resolve to outside working directory
- **Impact**: Low (CLI tool assumes trusted operator), but inconsistent with defense-in-depth

### NIT

**7. Inconsistent error handling for 403**
- **File**: `tests/attachment_download_command.rs:76`
- **Issue**: Test asserts exit code is "11 or 2" - unclear which is correct for 403
- **Note**: README says 11 is for auth errors (401/403), but test allows fallback to 2 (usage error)
- **Suggestion**: Clarify which exit code should be returned for 403

## Test quality

Spot-checked tests:

**1. `tests/attachment_download.rs::test_download_attachment_success`**
- **Depth**: GOOD - Tests end-to-end download flow
- **Assertions**: Checks success, file existence, and exact content match
- **Cleanup**: Properly removes test file
- **Verdict**: Passes non-trivial behavioral assertions

**2. `tests/attachment_list_command.rs::test_attachments_list_success`**
- **Depth**: GOOD - Tests full command through CLI
- **Assertions**: Checks both attachments appear in output, footer message shown
- **Coverage**: Verifies download hint message is present
- **Verdict**: Tests real user-visible behavior, not just internal state

**3. `tests/issue_show_footer.rs::test_issue_show_with_notes_and_attachments`**
- **Depth**: EXCELLENT - Tests interaction of two features
- **Assertions**: Both journal notes and attachment footer present
- **Note**: This test actually passes even though the query parameter bug prevents this combination in real usage (see BUG #2)
- **Verdict**: Good intent, but test may be passing for wrong reason

Overall test quality is strong with good coverage of success paths, error conditions, and integration scenarios.

## Suggested changes before merge

1. **[BLOCKING]** Fix clippy error in `src/commands/issue.rs:63`
2. **[BLOCKING]** Fix query parameter logic in `show()` to include both attachments and journals when `--notes` is used
3. **[MAJOR]** Implement atomic write pattern in `download_attachment()` to prevent partial files
4. **[MAJOR]** Add missing newline at end of `src/commands/issue.rs`
5. **[MINOR]** Consider improving default filename behavior or requiring `--output` flag
6. **[NIT]** Clarify exit code for 403 auth errors

Once BLOCKING and MAJOR issues are addressed, the feature will be ready for merge.