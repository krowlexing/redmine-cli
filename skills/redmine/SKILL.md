---
name: redmine
description: Operate Redmine from the command line - list, show, create, and update issues, list and download attachments, browse projects, users, and statuses. Use for any Redmine task when the redmine CLI is installed.
---

# redmine CLI

Agent-friendly command-line client for the Redmine REST API. Default output is tab-separated, machine-parseable, no color. Errors go to stderr with categorized exit codes.

## Setup

```bash
redmine config set --url https://redmine.example.com --key <API_KEY> --default-project <identifier>
redmine config show    # verify (api_key is masked)
```

Config precedence: CLI flags > environment (`REDMINE_URL`, `REDMINE_API_KEY`, `REDMINE_PROJECT`, `REDMINE_FORMAT`) > config file (`redmine config path`).

## Commands

```bash
redmine issue mine
redmine issue list [--assigned-to <id|me|name>] [--status <open|closed|name|id>] [--project <ident|id|->] [--limit N] [--sort field:dir] [--all]
redmine issue show <ID> [--notes]
redmine issue create --subject <S> [--project <ident|id|->] [--description <D>] [--assignee <id|me|name>] [--tracker <name|id>] [--priority <name|id>] [--status <name|id>]
redmine issue update <ID> [--status S] [--assignee A] [--subject S] [--description D] [--priority P] [--done 0-100] [--note <comment>]
redmine issue set-status <ID> <STATUS>
redmine issue assign <ID> <ASSIGNEE>
redmine issue close <ID>
redmine issue attachments list <ID>
redmine issue attachments download <ID> [--output <path>]
redmine project list
redmine project show <id|identifier>
redmine user list [--name <substr>]
redmine status
```

## Rules for agents

- Mutating commands (`create`, `update`, `set-status`, `assign`, `close`) are refused by default with exit code 6. This is intentional. Do not attempt to bypass it; ask the human to enable mutations on the host.
- Statuses, priorities, trackers, and assignees accept human names or ids; `me` resolves to the API key owner. Ambiguous names fail with exit code 4 and list candidates.
- `--format json` emits one compact JSON object per record; `--format pretty` renders a Markdown table for humans.
- Exit codes: 2 usage, 3 not found, 4 ambiguous name, 5 config, 6 mutation blocked, 10 other HTTP, 11 auth, 12 HTTP not found, 13 validation, 14 server, 15 network, 16 decode.
