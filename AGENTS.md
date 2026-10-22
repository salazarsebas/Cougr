# AGENTS.md

Instructions for AI coding agents (Codex, Cursor, Copilot, Claude Code, Gemini, and others) working on Cougr. If you are a person, this is also the shortest path to a pull request that gets merged.

## The one rule

Run `make check` from the repository root before every commit you push. Fix everything it reports and run it again until the last line says `All checks passed.` It runs the same checks as CI for the parts of the repository you changed. A pull request is only merged when every CI workflow passes.

- `make fix` applies the automatic fixes (formatting, docs site sync).
- `make check-all` runs every check.
- If a tool is missing, `make check` says how to install it. A skipped check still runs in CI.
- CI tests your branch merged into `main`, so keep it current: `git fetch upstream && git rebase upstream/main`. Add the remote once with `git remote add upstream https://github.com/salazarsebas/Cougr.git`.

Do not report a task as done, and do not open a pull request, while `make check` fails.

## Read your own diff before you push

Run `git diff upstream/main...HEAD --stat`, then read the full diff. Delete anything that is not part of the fix:

- Files the issue does not need.
- Generated output: `target/`, `*.wasm`, logs, coverage reports, build folders.
- Encoded or minified content. A source file must be readable source code, never base64 or one giant line.
- Editor, agent, or scratch files: plans, notes, TODO lists, `.cursor/`, `.aider*`.
- Placeholder code: `todo!()`, `unimplemented!()`, empty function bodies, commented-out blocks.

Write files with normal editor or file tools. Never paste an API response or encoded content into a source file.

## Scope

- One issue per pull request. Do what the issue asks and nothing more.
- You must be assigned to the issue before opening the pull request.
- Put `Closes #<issue>` in the pull request description.
- Commits follow Conventional Commits: `<type>(<scope>): <summary>`.

## Project conventions

[CLAUDE.md](CLAUDE.md) has the Cougr rules for Soroban code, `no_std`, the standards layer, and examples. [CONTRIBUTING.md](CONTRIBUTING.md) has the full contributor guide. Read the doc for the area you touch before writing code.

## When CI fails on your pull request

A bot comment on the pull request lists the failing jobs. Run `make check`, fix what it reports, and push to the same branch. Do not open a new pull request for the same issue. If a failure looks unrelated to your change, say so in a comment on the pull request.
