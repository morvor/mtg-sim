#!/usr/bin/env bash
# Mutation testing of the core engine files with cargo-mutants (see docs/MUTATION.md).
#
#   scripts/mutants.sh sba layers ...            # sample run, one file at a time
#   RERUN=1 scripts/mutants.sh sba               # re-run only the last run's missed/timeout
#                                                # mutants (by exact name), into <stem>-rerun
#   SAMPLE=1 scripts/mutants.sh trigger_timing   # every mutant instead of a 1-in-N sample
#
# Each file stem names crates/mtg-engine/src/<stem>.rs. Mutants run in a persistent copy of
# the tree (target/mutsrc, `--in-place` there) so every file reuses one incremental build;
# results land in target/mutants/<stem>/mutants.out and a summary line is printed.
# Sampling: cargo-mutants' round-robin shard 0/SAMPLE (default 10: mutants 0, 10, 20, ...
# in source order), except files with fewer than 60 mutants, which run in full.
# Machine limits (CLAUDE.md): one mutant at a time (implied by --in-place), CARGO_BUILD_JOBS=2,
# a global flock around every run.
set -euo pipefail
export CARGO_BUILD_JOBS=2
WT=$(git rev-parse --show-toplevel)
MS=$WT/target/mutsrc
OUT=$WT/target/mutants
SAMPLE=${SAMPLE:-10}
LOCK=${LOCK:-/tmp/cargo-heavy.lock}
mkdir -p "$MS" "$OUT"

for stem in "$@"; do
  # Fresh, unmutated source (an interrupted --in-place run may have left a mutation behind).
  find "$MS" -mindepth 1 -maxdepth 1 ! -name target -exec rm -rf {} +
  tar -C "$WT" --exclude=./target --exclude=./.git -cf - . | tar -C "$MS" -xf -
  file=crates/mtg-engine/src/$stem.rs
  n=$(cd "$MS" && cargo mutants --list -f "$file" | wc -l)
  k=$SAMPLE
  ((n < 60)) && k=1
  dest=$OUT/$stem
  args=(mutants --in-place -d "$MS" -f "$file" --timeout 300 --build-timeout 900
    --cap-lints true --test-package mtg-engine)
  if [[ -n "${RERUN:-}" ]]; then
    prev=$OUT/$stem/mutants.out
    dest=$OUT/$stem-rerun
    k=1
    while IFS= read -r name; do
      args+=(-F "^$(printf '%s' "$name" | sed 's/[][\\.^$*+?(){}|]/\\&/g')\$")
    done < <(cat "$prev/missed.txt" "$prev/timeout.txt" 2>/dev/null)
    args+=(--baseline skip)
  elif ((k > 1)); then
    args+=(--shard "0/$k" --sharding round-robin)
  fi
  args+=(-o "$dest")
  echo "== $stem: $n mutants, sample 1/$k: cargo ${args[*]}"
  rc=0
  flock "$LOCK" env CARGO_PROFILE_DEV_DEBUG=false cargo "${args[@]}" || rc=$?
  # 0: all caught, 2: some missed, 3: some timed out; anything else (usage error, baseline
  # failure, interrupted) stops the driver.
  if ((rc != 0 && rc != 2 && rc != 3)); then
    echo "== $stem: cargo mutants failed with exit code $rc"
    exit "$rc"
  fi
  mo=$dest/mutants.out
  c() { [[ -f "$mo/$1.txt" ]] && wc -l <"$mo/$1.txt" || echo 0; }
  echo "== $stem: caught $(c caught) missed $(c missed) timeout $(c timeout) unviable $(c unviable)"
done
