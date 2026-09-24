# redmine-cli

A command-line client for the Redmine REST API, designed to be **agent-friendly**: token-efficient output, no decorative symbols by default, machine-parseable formats.

## Install

```
cargo install --path .
```

The package is `redmine-cli`; it installs a single binary named **`redmine`**.

## Configure

Layered config (highest precedence wins): CLI flags → environment → config file.

```
redmine config set --url https://redmine.example.com --key <API_KEY> --default-project infra
redmine config show
redmine config path
```

Environment variables: `REDMINE_URL`, `REDMINE_API_KEY`, `REDMINE_PROJECT`, `REDMINE_FORMAT`.

## Output formats

`--format tab|pretty|json` (env `REDMINE_FORMAT`, default `tab`).

- **tab** — header line + `\t`-separated rows. No borders, no color. Default; best for LLM agents.
- **pretty** — Markdown-style pipe table with alignment. For humans.
- **json** — compact single-line JSON per record. For structured/scripted consumption.

Errors go to stderr; exit codes are non-zero and categorized (see below).

`--verbose` makes HTTP errors print the request method and full response body (normally truncated to 200 chars) — useful for debugging API failures.

## Mutations are blocked by default

For safety, all mutating commands (`issue create`, `update`, `set-status`, `assign`, `close`) are **refused by default** with exit code 6 (`mutable operations are blocked. requires human intervention`). Read-only commands are unaffected.

To enable mutations, set one of:

- `mutable = true` in the config file (find it with `redmine config path`)
- the environment variable `REDMINE_ALLOW_MUTATIONS` to `1`, `true`, `yes` or `on` (overrides the config file)

There is no CLI flag to unlock mutations.

## Shell completions

```
redmine completion bash    # also: zsh, fish, elvish, powershell
```

Pipe to your shell's completion directory, e.g. `redmine completion zsh > _redmine`.

## Commands

### Issues

```
redmine issue mine                           # assigned to me
redmine issue list [--assigned-to <id|me|name>] [--status <open|closed|name|id>]
                       [--project <ident|id|->] [--limit N] [--sort field:dir] [--all]
redmine issue show <ID> [--notes]            # detail; --notes adds journal history
redmine issue update <ID> [fields...]        # partial update (any subset)
redmine issue create --subject <S> [--project <ident|id|->]
                          [--description <D>] [--assignee <id|me|name>]
                          [--tracker <name|id>] [--priority <name|id>] [--status <name|id>]
redmine issue set-status <ID> <STATUS>
redmine issue assign <ID> <ASSIGNEE>
redmine issue close <ID>
redmine issue attachments list <ID>
redmine issue attachments download <ID> [--output <path>]
```

`issue show` prints a footer with the attachment count and a listing hint when the issue has attachments. `download` without `--output` saves the file under its actual filename from the server.

Update fields: `--status --assignee --subject --description --priority --done <0-100> --note <comment>`.

Name resolution: status/priority/tracker/assignee accept human names or ids; `me` resolves to the API key's user. Ambiguous names error with candidates.

### Reference

```
redmine project list
redmine project show <id|identifier>
redmine user list [--name <substr>]
redmine status                              # list issue statuses
```

## Use cases

| Task | Command |
|---|---|
| list assigned issues | `redmine issue mine` |
| show description by id | `redmine issue show 1234` |
| change status | `redmine issue set-status 1234 "In Progress"` |
| change assignee | `redmine issue assign 1234 me` |
| create issue | `redmine issue create --project infra --subject "Fix build" --description "..."` |
| list attachments | `redmine issue attachments list 1234` |
| download attachment | `redmine issue attachments download 1234` |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 2 | usage error |
| 3 | not found |
| 4 | ambiguous name |
| 5 | config error |
| 6 | mutation blocked (needs human approval) |
| 10 | other HTTP error |
| 11 | auth error (401/403) |
| 12 | not found HTTP (404) |
| 13 | validation error (422) |
| 14 | server error (5xx) |
| 15 | network error |
| 16 | decode error |

## Examples

```
$ redmine issue mine
id	project	tracker	status	priority	assignee	subject	updated
1234	infra	Bug	In Progress	High	Alice Smith	Fix the build	2026-07-05 10:00:00

$ redmine --format pretty issue mine
| id   | project | tracker | status      | priority | assignee    | subject       | updated             |
|------|---------|---------|-------------|----------|-------------|---------------|---------------------|
| 1234 | infra   | Bug     | In Progress | High     | Alice Smith | Fix the build | 2026-07-05 10:00:00 |

$ redmine issue attachments list 1234
id	filename	size	content_type	author	created_on
17	build.log	2.3 KB	text/plain	Alice Smith	2026-07-05 10:00:00
Use `redmine issue attachments download <attachment-id>` to download attachment into cwd. Check --help for more flags.

$ redmine issue show 1234
id	1234
project	infra
status	In Progress
...
DESCRIPTION
The CI build fails...
This issue has 1 attachment. Run `redmine issue attachments list 1234` to list them.
```

## Project layout

```
src/
  lib.rs         crate root (re-exports modules for tests)
  main.rs        dispatch + exit codes
  cli.rs         clap derive definitions
  config.rs      layered config (flags > env > file)
  client.rs      blocking Redmine HTTP client
  resolve.rs     name ↔ id resolution + caching
  output.rs      tab/pretty/json renderers (generic over writer)
  error.rs       typed errors + verbose flag
  models/        serde models (issue, project, user, common)
  commands/      issue, project, user, config, completion
```

## Tests

```
cargo test
```

Integration tests use a built-in synchronous mock HTTP server (`tests/common/mod.rs`) — no external test server required. Covers list/show/create/update flows, name resolution, ambiguous-name errors, HTTP error mapping, output modes, and verbose error formatting.

## License

MIT
