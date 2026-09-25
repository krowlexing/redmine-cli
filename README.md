# redmine-cli

A command-line client for the Redmine REST API, designed to be **agent-friendly**: token-efficient output, no decorative symbols by default, machine-parseable formats.

```
redmine issue mine                           # assigned to me
redmine issue list --project infra --status open
redmine issue show 1234 --notes              # description + history
redmine issue attachments download 1234
redmine project list
```

Name resolution: status/priority/tracker/assignee accept human names or ids; `me` resolves to the API key's user. Run `redmine <command> --help` for all flags.

## Install

### Pre-built binary

Download `redmine` for Linux or `redmine.exe` for Windows from [GitHub releases](https://github.com/<owner>/redmine-cli/releases), make it executable, and put it on your `PATH`.

A `.sha256` checksum is published next to each binary.

### From source

```
cargo install --git https://github.com/<owner>/redmine-cli.git
```

The package is `redmine-cli`; it installs a single binary named **`redmine`**.

To make coding agents discover usage instructions:

```
redmine skill install
```

## Configure

Layered config (highest precedence wins): CLI flags → environment → config file.

```
redmine config set --url https://redmine.example.com --key <API_KEY> --default-project infra
redmine config show
redmine config path
```

Environment variables: `REDMINE_URL`, `REDMINE_API_KEY`, `REDMINE_PROJECT`, `REDMINE_FORMAT`.

## Mutations are blocked by default

For safety, all mutating commands (`issue create`, `update`, `set-status`, `assign`, `close`) are **refused by default** with exit code 6 (`mutable operations are blocked. requires human intervention`). Read-only commands are unaffected.

To enable mutations, set one of:

- `mutable = true` in the config file (find it with `redmine config path`)
- the environment variable `REDMINE_ALLOW_MUTATIONS` to `1`, `true`, `yes` or `on` (overrides the config file)

There is no CLI flag to unlock mutations.

## Output formats

`--format tab|pretty|json` (env `REDMINE_FORMAT`, default `tab`).

- **tab** — header line + `\t`-separated rows. No borders, no color. Default; best for LLM agents.
- **pretty** — Markdown-style pipe table with alignment. For humans.
- **json** — compact single-line JSON per record. For structured/scripted consumption.

Errors go to stderr; exit codes are non-zero and categorized (see below).

`--verbose` makes HTTP errors print the request method and full response body (normally truncated to 200 chars) — useful for debugging API failures.

## Shell completions

```
redmine completion bash    # also: zsh, fish, elvish, powershell
```

Pipe to your shell's completion directory, e.g. `redmine completion zsh > _redmine`.

## Agent skill

Coding agents can discover usage instructions as a skill:

```
redmine skill install    # write SKILL.md to ~/.agents/skills/redmine
redmine skill show       # print the skill content for inspection
```

The source file is `skills/redmine/SKILL.md`, embedded into the binary at build time; copy it manually if needed. Re-running install overwrites the file.

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

$ redmine issue show 1234
id	1234
project	infra
status	In Progress
...
DESCRIPTION
The CI build fails...
```

## License

MIT
