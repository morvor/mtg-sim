#!/usr/bin/env python3
"""Check the round-3 card-support plan: every unsupported text is placed exactly once.

The input snapshot is round3/baseline.tsv (`mtg-tools unsupported --paper --by-card` when
the plan was made: one card<TAB>text line per unsupported text of a paper card, numbers as
N, the card's own name as ~). Every distinct text in it must appear in exactly one of:

- a round-3 item list, items/cards3-*.texts.tsv (including cards3-manual-defs);
- round3/inflight.tsv (owned by a round-2 item still in progress);
- round3/pending-recheck.tsv (waiting for an in-progress item's construct);
- round3/deferred.tsv (out of scope under docs/DEFERRED.md);

and those files must not list texts that aren't in the snapshot. The card counts in the
lists must match the snapshot. Exit status 1 on any problem.

  check_plan.py           # check the plan files against the snapshot
  check_plan.py --live    # also compare the snapshot with the current release mtg-tools
                          # (texts supported since are reported; a live text that no list
                          # places is a problem, so the plan is checked against a fresh list)
"""
import collections
import glob
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
AGENTS = os.path.dirname(HERE)
ROOT = os.path.dirname(os.path.dirname(AGENTS))


def read_tsv(path):
    with open(path, encoding="utf-8") as f:
        lines = [l.rstrip("\n") for l in f if l.strip()]
    head = lines[0].split("\t")
    return [dict(zip(head, l.split("\t"))) for l in lines[1:]]


def main():
    problems = []
    base = read_tsv(os.path.join(HERE, "baseline.tsv"))
    cards_of = collections.defaultdict(set)
    for r in base:
        cards_of[r["text"]].add(r["card"])
    sources = {}
    for p in sorted(glob.glob(os.path.join(AGENTS, "items", "cards3-*.texts.tsv"))):
        sources[os.path.relpath(p, AGENTS)] = read_tsv(p)
    for name in ("inflight.tsv", "pending-recheck.tsv", "deferred.tsv"):
        sources["round3/" + name] = read_tsv(os.path.join(HERE, name))
    seen = collections.defaultdict(list)
    for src, rows in sources.items():
        for r in rows:
            t = r["text"]
            seen[t].append(src)
            if t not in cards_of:
                problems.append(f"{src}: text not in the snapshot: {t[:100]}")
                continue
            if int(r["cards"]) != len(cards_of[t]):
                problems.append(f"{src}: card count {r['cards']} != {len(cards_of[t])}: {t[:80]}")
            if r.get("example_card") and r["example_card"] not in cards_of[t]:
                problems.append(f"{src}: example card {r['example_card']} doesn't have: {t[:80]}")
    for t in cards_of:
        if t not in seen:
            problems.append(f"unplaced: {t[:100]} [{sorted(cards_of[t])[0]}]")
        elif len(seen[t]) > 1:
            problems.append(f"placed {len(seen[t])} times ({', '.join(seen[t])}): {t[:80]}")
    counts = collections.Counter(src for t in cards_of for src in seen.get(t, [])[:1])
    print(f"snapshot: {len(base)} card-text lines, {len(cards_of)} distinct texts, "
          f"{len({r['card'] for r in base})} cards")
    for src in sorted(sources):
        print(f"  {counts[src]:5d}  {src}")
    if "--live" in sys.argv:
        tools = os.path.join(ROOT, "target", "release", "mtg-tools")
        out = subprocess.run([tools, "unsupported", "--paper", "--by-card"], capture_output=True,
                             text=True, check=True).stdout
        live = {tuple(l.split("\t", 1)) for l in out.splitlines() if "\t" in l}
        snap = {(r["card"], r["text"]) for r in base}
        print(f"live: {len(live)} card-text lines; supported since the snapshot: "
              f"{len(snap - live)}; new since: {len(live - snap)}")
        for c, t in sorted(live - snap):
            print(f"  NEW {c}: {t[:100]}")
            if t not in seen:
                problems.append(f"live text placed nowhere: {t[:100]} [{c}]")
    for p in problems:
        print("PROBLEM", p)
    if problems:
        print(f"FAILED: {len(problems)} problems")
        sys.exit(1)
    print("OK: every unsupported text is placed exactly once")


if __name__ == "__main__":
    main()
