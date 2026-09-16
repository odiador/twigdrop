# Plan: Technical Debt Cleanup

Audience: whoever implements this (may not be the person who wrote it). Each item is independent — do them in order, but a partial run is still valuable.

## 1. Repo hygiene (DONE — 2026-09-16)

- [x] Removed `scratch_debug` (494KB compiled binary) and `scratch_debug.rs` from the entire `main` history via `git filter-repo` + force-push.
- [x] Added `scratch_debug` / `scratch_debug.rs` to `.gitignore`.
- **Action if you're a collaborator**: re-clone the repo, or run:
  ```
  git fetch origin
  git reset --hard origin/main
  ```
  Your old `main` has diverged (different commit hashes) — do not try to merge/rebase your old local branch onto the new one, just reset.
- Other remote branches (`architecture-refactor`, `dev/improvements`, `feat/animations-and-previews`) were NOT rewritten — still carry the old commits with the binary. Rewrite them the same way before merging, or accept the debt on those branches specifically.

## 2. Test coverage (no tests exist)

Current state: `src/git/status.rs` has a `#[cfg(test)]` module with zero `#[test]` functions inside — it's a decorative stub.

Given this tool performs destructive git operations (branch deletion, stash apply, rebase), untested code is the most expensive debt here.

Priority order:
1. `src/git/` — pure logic functions (parsing `git log`/`git status` output, merge-tree conflict detection). These are the easiest to unit test since they don't need a live terminal, and the highest value since bugs here cause data loss.
2. `src/actions/` — business logic orchestration. Test with a temp git repo fixture (`tempfile` crate + `git2::Repository::init`).
3. Skip trying to test `src/ui/` and `src/handlers/` directly — TUI rendering/input is better covered by manual testing or snapshot tests later. Not a priority now.

- [x] Extracted and unit tested pure git log & status parsers (`parse_ahead_behind`, `parse_branch_metadata_line`, `parse_stashed_branch_line`, `parse_commit_tree_line`).
- [x] Extracted and unit tested destructive branch pruning filter (`get_prunable_branches` in `src/actions/commands.rs`), verifying active branches, stashed branches, and branches with unique commits are never pruned.
- [ ] Integration tests with temp git repo fixture (`tempfile` + `git2::Repository::init`) for live git commands.

## 3. God files — split by domain (DONE — 2026-09-16)

- [x] `src/ui/screens.rs` (1983 lines) → split into `src/ui/screens/{branches,commits,stashes,files,modals,diff,mod}.rs`.
- [x] `src/handlers/keyboard.rs` (1625 lines) → split into `src/handlers/keyboard/{branches,commits,stashes,files,modals,diff,mod}.rs`.

## 4. Small fixes (DONE — 2026-09-16)

- [x] Fix clippy warning at `src/ui/screens.rs:757` — `.enumerate()` with discarded index replaced with iterator over actions.
- [x] Verified `src/git/commands.rs` has zero `.expect()` calls in production code and returns `Result`.
- [x] Unified layout constants duplicated between screens and mouse into `src/ui/layout.rs` (`calculate_diff_layout` and `calculate_modal_rect`).
- [x] Removed panic triggers (`.expect()`, `.unwrap()`) from `src/utils/config.rs` using safe fallback and `entry` API, covered by unit tests.
- [x] Zero clippy warnings across binary and all targets (`cargo clippy --all-targets`).

## Out of scope for this plan

Feature work from `docs/IMPROVEMENTS.md` (branch protection, backups, search mode, etc.) — that's roadmap, not debt. Don't mix the two in the same PR.
