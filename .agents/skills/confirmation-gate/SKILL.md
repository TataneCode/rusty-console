---
name: confirmation-gate
description: Mandatory confirmation gate before any write action (files, branches, commits, PRs) on this repository. Apply before starting any task that modifies the repository, whatever the entry point (Vibe Code, Vibe CLI, Mistral CLI).
---

# Confirmation Gate

## Purpose

Protect this repository from unsolicited implementation. The agent must never
start a write action on its own initiative after receiving a user request. It
must first present what it intends to do and wait for the user's explicit
confirmation.

## Scope

This skill applies to **every entry point** (Vibe Code, Vibe CLI, Mistral CLI)
and to **every write action** on this repository:

- Creating, renaming, or deleting files or directories
- Modifying existing files
- Creating or switching to a branch
- Committing
- Pushing to the remote
- Opening, updating, or closing a pull request / merge request
- Merging
- Any other operation that changes the state of the repository or its remote

Pure read-only operations are **exempt** and do not require confirmation:

- Reading files, searching the codebase
- Running `cargo check`, `cargo test`, `cargo fmt --check`, `cargo clippy`,
  coverage, or any other verification command that does not write to the repo
  (build artifacts in `target/` and editor state are tolerated)

## Procedure

1. **Analyze** the request and inspect the repository (read-only).
2. **Plan** the change: list the files to create or modify, the commands to
   run, and the intended Git actions (branch name, commit, push, PR/MR).
3. **Present the plan** to the user in a concise, structured form.
4. **Ask for explicit confirmation** (a direct question, or the repository's
   preferred confirmation mechanism).
5. **Wait** for the answer. Do not proceed, do not "assume" approval.
6. **Only on explicit approval**, execute the plan exactly as presented. Any
   scope change discovered mid-course requires a new confirmation.

## Decision Rules

- Silence, an ambiguous answer, or absence of response means **do not
  proceed**.
- A confirmation is valid only for the exact plan presented; it does not
  extend to follow-up actions (e.g., approval to commit is not approval to
  push or open a PR).
- Read-only verification after an approved change (tests, lint, build) does
  not require a new confirmation, but publishing the result (commit, push,
  PR) does.
- If the user explicitly pre-approves a scope in their request (e.g. "commit
  and push directly"), that explicit instruction satisfies this gate for the
  stated scope only.
