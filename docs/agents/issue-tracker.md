# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues. Use the `gh` CLI for all operations. This repo currently has no remote configured; once the GitHub remote is added, infer the repo from `git remote -v`.

## Conventions

- Create an issue: `gh issue create --title "..." --body "..."`
- Read an issue: `gh issue view <number> --comments`
- List issues: `gh issue list --state open --json number,title,body,labels,comments`
- Comment on an issue: `gh issue comment <number> --body "..."`
- Apply or remove labels: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- Close: `gh issue close <number> --comment "..."`

## Pull requests as a triage surface

PRs as a request surface: no.

External pull requests should not be pulled into the issue triage queue. Treat PRs as normal code-review work unless this file is changed later.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.

## Wayfinding operations

Used by `/wayfinder`. The map is a single issue labelled `wayfinder:map`, holding Notes, Decisions-so-far, and Fog. Child tickets are GitHub issues linked to the map through GitHub sub-issues when available; otherwise, add them to a task list in the map body and include `Part of #<map>` at the top of the child body.

Use GitHub native issue dependencies for blocking edges when available. If unavailable, fall back to a `Blocked by: #<n>, #<n>` line at the top of the child body.
