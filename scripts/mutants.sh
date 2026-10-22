#!/usr/bin/env bash
# Mutation testing of the core engine files with cargo-mutants (see docs/MUTATION.md).
#
#   scripts/mutants.sh sba layers ...            # sample run, one file at a time
#   RERUN=1 scripts/mutants.sh sba               # re-run only the last run's missed/timeout
#                                                # mutants, into <stem>-rerun
#   RERUN=1 ONLY='mana.rs:1019' scripts/mutants.sh mana   # only the survivors matching
#   SAMPLE=1 scripts/mutants.sh trigger_timing   # every mutant instead of a 1-in-N sample
#
# Each file stem names crates/mtg-engine/src/<stem>.rs. Mutants run in a persistent copy of
# the committed tree (target/mutsrc, `--in-place` there) so every file reuses one incremental build;
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
  # Fresh, unmutated source of the committed tree (an interrupted --in-place run may have
  # left a mutation behind; uncommitted work in progress must not break the baseline).
  # -m: fresh mtimes, so cargo rebuilds rather than reusing the last mutant's binaries.
  find "$MS" -mindepth 1 -maxdepth 1 ! -name target -exec rm -rf {} +
  git -C "$WT" archive HEAD | tar -C "$MS" -xmf -
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
    # Each survivor is found again in today's list of the file's mutants: the same mutation
    # in the same function whose diff (the mutated line and the lines around it, from the
    # earlier run's mutants.out/diff/) matches best, then the same column and the nearest
    # line (edits elsewhere in the file may have moved it; line distance alone can pick
    # a neighbouring identical mutation).
    cat "$prev/missed.txt" "$prev/timeout.txt" 2>/dev/null >"$OUT/$stem.survivors"
    (cd "$MS" && cargo mutants --list --json --diff -f "$file") >"$OUT/$stem.list.json"
    names=()
    while IFS= read -r name; do
      # ONLY=<regex>: re-run just the survivors whose name matches.
      [[ -z "${ONLY:-}" || "$name" =~ $ONLY ]] && names+=("$name")
    done < <(python3 - "$OUT/$stem.survivors" "$OUT/$stem.list.json" "$prev/diff" <<'PY'
import difflib, json, os, re, sys
pat = re.compile(r"^(.*?):(\d+):(\d+): (.*)$")
def body(diff):
    # The hunk without its file and @@ header lines (which carry line numbers).
    return [l for l in diff.splitlines() if not l.startswith(("---", "+++", "@@"))]
now = []
for m in json.load(open(sys.argv[2])):
    g = pat.match(m["name"])
    if g:
        now.append((*g.groups(), body(m["diff"])))
for line in open(sys.argv[1]).read().splitlines():
    m = pat.match(line)
    if not m:
        continue
    f, ln, col, rest = m.groups()
    cands = [c for c in now if c[0] == f and c[3] == rest]
    if not cands:
        print(f"no longer generated: {line}", file=sys.stderr)
        continue
    old = os.path.join(sys.argv[3], f"{f.replace('/', '__')}_line_{ln}_col_{col}.diff")
    ob = body(open(old).read()) if os.path.exists(old) else None
    def score(c):
        sim = difflib.SequenceMatcher(None, ob, c[4]).ratio() if ob is not None else 0
        return (-sim, c[2] != col, abs(int(c[1]) - int(ln)))
    best = min(cands, key=score)
    print(f"{best[0]}:{best[1]}:{best[2]}: {best[3]}")
PY
    )
    if ((${#names[@]} == 0)); then
      echo "== $stem: no survivors to re-run"
      continue
    fi
    # cargo-mutants 27.1 applies neither -F nor -E to "delete field" mutants (it would
    # test every one of them), so each survivor is applied and tested here, the way
    # cargo-mutants does it: build (unviable if that fails), then the mtg-engine tests
    # (missed if they pass, timeout after 300 s, caught otherwise).
    mo=$dest/mutants.out
    # A full re-run starts afresh; with ONLY, the other survivors' results are kept.
    [[ -z "${ONLY:-}" ]] && rm -rf "$dest"
    mkdir -p "$mo"
    touch "$mo"/{caught,missed,timeout,unviable}.txt
    echo "== $stem: re-running ${#names[@]} survivors"
    for name in "${names[@]}"; do
      python3 - "$OUT/$stem.list.json" "$name" >"$OUT/cur.diff" <<'PY'
import json, sys
print(next(m["diff"] for m in json.load(open(sys.argv[1])) if m["name"] == sys.argv[2]), end="")
PY
      cp "$MS/$file" "$OUT/cur.orig"
      if ! patch -s "$MS/$file" <"$OUT/cur.diff"; then
        cp "$OUT/cur.orig" "$MS/$file"
        echo "$name" >>"$mo/unviable.txt"
        echo "UNVIABLE (patch failed) $name"
        continue
      fi
      outcome=$(flock "$LOCK" env CARGO_PROFILE_DEV_DEBUG=false bash -c '
        cd "$1" || exit
        timeout -k 10 900 cargo test -p mtg-engine --no-run >/dev/null 2>&1 || { echo unviable; exit; }
        timeout -k 10 300 cargo test -p mtg-engine >/dev/null 2>&1
        case $? in 0) echo missed ;; 124 | 137) echo timeout ;; *) echo caught ;; esac' _ "$MS")
      cp "$OUT/cur.orig" "$MS/$file"
      for o in caught missed timeout unviable; do
        grep -vxF -- "$name" "$mo/$o.txt" >"$mo/$o.tmp" || true
        mv "$mo/$o.tmp" "$mo/$o.txt"
      done
      echo "$name" >>"$mo/$outcome.txt"
      echo "${outcome^^} $name"
    done
    c() { wc -l <"$mo/$1.txt"; }
    echo "== $stem: caught $(c caught) missed $(c missed) timeout $(c timeout) unviable $(c unviable)"
    continue
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
