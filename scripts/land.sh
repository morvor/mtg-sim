#!/usr/bin/env bash
# Land the current wip/* branch onto the PR branch on origin. Several machines may be
# landing at the same time, so this only ever fast-forwards origin.
#
# Run from a checkout or worktree on a wip/<slug> branch with everything committed.
# 1. fetches origin's PR branch and merges it into this branch (on conflict: stops; resolve
#    keeping both sides' behavior, commit, re-run),
# 2. builds and tests the whole workspace (on failure: stops; fix, commit, re-run),
# 3. pushes this branch to origin as the PR branch, fast-forward only. If another landing
#    got there first, loops back to step 1.
# 4. fast-forwards a clean local checkout of the PR branch, if there is one.
# Exit codes: 0 landed, 2 usage, 3 merge conflict, 4 build or test failure, 6 gave up.
set -uo pipefail

main() {
  local PR=${PR_BRANCH:-claude/mtg-rules-engine-lkr27h}
  local BR
  BR=$(git branch --show-current)
  [[ "$BR" == wip/* ]] || { echo "ERROR: current branch '$BR' is not a wip/* branch"; exit 2; }
  [[ -z "$(git status --porcelain --untracked-files=no)" ]] || {
    echo "ERROR: uncommitted changes; commit first"; git status --short | head; exit 2; }
  local LOG
  LOG=$(mktemp)
  for attempt in $(seq 1 12); do
    fetch "$PR" || { echo "ERROR: cannot fetch origin/$PR"; exit 6; }
    if ! git merge-base --is-ancestor "origin/$PR" HEAD; then
      echo "== merging origin/$PR into $BR"
      if ! git merge --no-edit "origin/$PR"; then
        echo "CONFLICT: resolve the conflicted files keeping BOTH sides' behavior,"
        echo "run cargo fmt, 'git add' them, 'git commit --no-edit', then re-run this script."
        exit 3
      fi
    fi
    echo "== cargo build --workspace --all-targets"
    if ! cargo build --workspace --all-targets >"$LOG" 2>&1; then
      grep -E "^error" -A8 "$LOG" | head -80
      echo "BUILD FAILED: fix, commit, re-run."
      exit 4
    fi
    grep -E "^warning" "$LOG" | sort | uniq -c | sort -rn | head -10
    echo "== cargo test --workspace"
    if ! cargo test --workspace >"$LOG" 2>&1; then
      grep -E "panicked|FAILED|^error|failures:" -A3 "$LOG" | head -80
      echo "TESTS FAILED: fix, commit, re-run."
      exit 4
    fi
    grep -E "^test result" "$LOG" | awk '{s+=$4; f+=$6} END {print "tests passed:", s, "failed:", f}'
    for i in 1 2 3 4; do
      if git push -q origin "HEAD:refs/heads/$PR" 2>/dev/null; then
        update_local "$PR"
        local root
        root=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
        if [[ -d "$root/.claude" ]]; then
          echo "$(date -u +%FT%TZ) $BR $(git rev-parse --short HEAD)" >> "$root/.claude/landed.log"
        fi
        echo "LANDED $(git rev-parse --short HEAD) on $PR"
        exit 0
      fi
      fetch "$PR" || true
      # origin moved: merge it and test again.
      git merge-base --is-ancestor "origin/$PR" HEAD || break
      sleep $((2 ** i))  # network trouble: retry the push
    done
    echo "== origin/$PR moved while testing; merging again"
    sleep $((attempt * 3))
  done
  echo "ERROR: could not land after several attempts"
  exit 6
}

fetch() {
  for i in 1 2 3 4; do
    git fetch -q origin "$1" 2>/dev/null && git update-ref "refs/remotes/origin/$1" FETCH_HEAD && return 0
    sleep $((2 ** i))
  done
  return 1
}

# Fast-forward the local PR branch to what was just pushed: in the worktree that has it
# checked out if that worktree is clean, or the bare ref if no worktree has it.
update_local() {
  local PR=$1 wt c
  c=$(git rev-parse HEAD)
  wt=$(git worktree list --porcelain | awk -v b="branch refs/heads/$PR" '/^worktree /{w=substr($0, 10)} $0 == b {print w}')
  if [[ -n "$wt" ]]; then
    if [[ -z "$(git -C "$wt" status --porcelain --untracked-files=no)" ]]; then
      git -C "$wt" merge --ff-only -q "$c" 2>/dev/null || true
    fi
  elif git show-ref -q --verify "refs/heads/$PR" && git merge-base --is-ancestor "$PR" "$c"; then
    git branch -f "$PR" "$c" 2>/dev/null || true
  fi
}

main "$@"
