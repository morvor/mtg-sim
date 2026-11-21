#!/usr/bin/env python3
"""Check the round-6 card-support plan: every unsupported text is placed exactly once.

Unlike round 5's check, this one always regenerates the live list: it runs the release
`mtg-tools unsupported --paper --by-card` of this checkout (target/release/mtg-tools; build it
first with `cargo build --release -p mtg-tools`) and fails on any live text that is not placed
in exactly one of:

- a round-6 item list, items/cards6-*.texts.tsv (including cards6-manual-defs);
- round6/deferred.tsv (out of scope under docs/DEFERRED.md; rows whose category starts with
  "digital version" are digital versions the paper rule leaves out, recorded but not counted).

It also checks the lists against the snapshot round6/baseline.tsv (the live list when the plan
was made; one card<TAB>text line per unsupported text of a paper card, numbers as N, the card's
own name as ~): every listed text must be in the snapshot, card counts and example cards must
match it, every list must have a brief next to it, and the work-item table of
card-support-plan-6.md must match the lists (texts, paper cards touched, cards with all their
unsupported texts in the item). Texts of the snapshot that compile since are reported (not a
problem: a landed item's work), and so is a list row whose text is no longer live.

Exit status 1 on any problem, including a live text placed nowhere or twice, or a release
binary older than the newest commit touching crates/ (rebuild it; `--stale-ok` skips that
check).

  check_plan.py              # live check + plan files against the snapshot
  check_plan.py --stale-ok   # don't fail when the binary predates the last crates/ commit
"""
import collections
import glob
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
AGENTS = os.path.dirname(HERE)
ROOT = os.path.dirname(os.path.dirname(AGENTS))
TOOLS = os.path.join(ROOT, "target", "release", "mtg-tools")
INTEGRATION = "origin/claude/mtg-rules-engine-lkr27h"


def read_tsv(path):
    with open(path, encoding="utf-8") as f:
        lines = [l.rstrip("\n") for l in f if l.strip()]
    head = lines[0].split("\t")
    rows = []
    for n, l in enumerate(lines[1:], 2):
        cells = l.split("\t")
        if len(cells) != len(head):
            raise SystemExit(f"{path}:{n}: {len(cells)} columns, expected {len(head)}")
        rows.append(dict(zip(head, cells)))
    return rows


def live_list():
    if not os.path.exists(TOOLS):
        raise SystemExit(f"no release mtg-tools at {TOOLS}: cargo build --release -p mtg-tools")
    out = subprocess.run([TOOLS, "unsupported", "--paper", "--by-card"], capture_output=True,
                         text=True, check=True).stdout
    return [tuple(l.split("\t", 1)) for l in out.splitlines() if "\t" in l]


def binary_is_stale():
    """True when the release binary is older than the newest commit touching crates/."""
    log = subprocess.run(["git", "-C", ROOT, "log", "-1", "--format=%ct", "--", "crates"],
                         capture_output=True, text=True).stdout.strip()
    return bool(log) and os.path.getmtime(TOOLS) < int(log)


def plan_table():
    """{item: (texts, touched, all_here)} from the work-item table of card-support-plan-6.md."""
    path = os.path.join(AGENTS, "card-support-plan-6.md")
    out = {}
    if not os.path.exists(path):
        return out
    for line in open(path, encoding="utf-8"):
        m = re.match(r"^\|\s*\d+\s*\|\s*(cards6-[a-z0-9-]+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)",
                     line)
        if m:
            out[m.group(1)] = tuple(int(m.group(k)) for k in (2, 3, 4))
    return out


def main():
    problems = []
    base = read_tsv(os.path.join(HERE, "baseline.tsv"))
    cards_of = collections.defaultdict(set)
    for r in base:
        cards_of[r["text"]].add(r["card"])
    sources = {}
    for p in sorted(glob.glob(os.path.join(AGENTS, "items", "cards6-*.texts.tsv"))):
        sources[os.path.relpath(p, AGENTS)] = read_tsv(p)
        if not os.path.exists(p[:-len(".texts.tsv")] + ".md"):
            problems.append(f"{os.path.relpath(p, AGENTS)}: no brief next to it")
    sources["round6/deferred.tsv"] = read_tsv(os.path.join(HERE, "deferred.tsv"))
    seen = collections.defaultdict(list)
    for src, rows in sources.items():
        for r in rows:
            t = r["text"]
            if src == "round6/deferred.tsv" and r.get("category", "").startswith("digital version"):
                if r["example_card"] in cards_of.get(t, set()):
                    problems.append(f"{src}: {r['example_card']} is a paper card: {t[:80]}")
                continue
            seen[t].append(src)
            if t not in cards_of:
                problems.append(f"{src}: text not in the snapshot: {t[:100]}")
                continue
            if int(r["cards"]) != len(cards_of[t]):
                problems.append(f"{src}: card count {r['cards']} != {len(cards_of[t])}: {t[:80]}")
            if r.get("example_card") and r["example_card"] not in cards_of[t]:
                problems.append(f"{src}: example card {r['example_card']} doesn't have: {t[:80]}")
    for t, srcs in seen.items():
        if len(srcs) > 1:
            problems.append(f"placed {len(srcs)} times ({', '.join(srcs)}): {t[:80]}")
    for t in cards_of:
        if t not in seen:
            problems.append(f"snapshot text placed nowhere: {t[:100]} [{sorted(cards_of[t])[0]}]")

    # The live list from the current binary: every live text placed exactly once.
    if binary_is_stale():
        msg = "the release mtg-tools is older than the last commit touching crates/: rebuild it"
        if "--stale-ok" in sys.argv:
            print("WARNING", msg)
        else:
            problems.append(msg)
    live = live_list()
    live_texts = collections.defaultdict(set)
    for c, t in live:
        live_texts[t].add(c)
    for t, cs in sorted(live_texts.items()):
        n = len(seen.get(t, []))
        if n != 1:
            problems.append(f"live text placed {n} times: {t[:100]} [{sorted(cs)[0]}]")
    snap = {(r["card"], r["text"]) for r in base}
    lv = set(live)
    owner = {t: s[0] for t, s in seen.items()}
    print(f"snapshot: {len(base)} card-text lines, {len(cards_of)} distinct texts, "
          f"{len({r['card'] for r in base})} cards")
    print(f"live: {len(lv)} card-text lines, {len(live_texts)} distinct texts, "
          f"{len({c for c, _ in lv})} cards; supported since the snapshot: {len(snap - lv)}; "
          f"new since: {len(lv - snap)}")
    for c, t in sorted(snap - lv):
        print(f"  SUPPORTED {c}: {t[:90]} [{owner.get(t, '?')}]")
    for c, t in sorted(lv - snap):
        print(f"  NEW {c}: {t[:100]} [{owner.get(t, 'placed nowhere')}]")

    # Per-list counts (over the snapshot), and the plan's item table.
    card_texts = collections.defaultdict(set)
    for r in base:
        card_texts[r["card"]].add(r["text"])
    print("  texts  touched  all-here  live  list")
    table = plan_table()
    for src in sorted(sources):
        ts = {t for t, o in owner.items() if o == src}
        touched = {c for c, tt in card_texts.items() if tt & ts}
        alone = {c for c, tt in card_texts.items() if tt <= ts}
        still = len(ts & set(live_texts))
        print(f"  {len(ts):5d}  {len(touched):7d}  {len(alone):8d}  {still:4d}  {src}")
        m = re.match(r"items/(cards6-[a-z0-9-]+)\.texts\.tsv$", src)
        if m and table:
            got = (len(ts), len(touched), len(alone))
            if m.group(1) not in table:
                problems.append(f"card-support-plan-6.md: no table row for {m.group(1)}")
            elif table[m.group(1)] != got:
                problems.append(f"card-support-plan-6.md: {m.group(1)} row says "
                                f"{table[m.group(1)]}, the list gives {got} (texts, touched, all here)")
    if not table:
        problems.append("card-support-plan-6.md: no work-item table found")

    # Which cards6 items have commits, here or on the fetched integration branch.
    refs = ["HEAD"] + [r for r in (INTEGRATION,) if subprocess.run(
        ["git", "-C", ROOT, "rev-parse", "--verify", "-q", r], capture_output=True).returncode == 0]
    for src in sorted(sources):
        m = re.match(r"items/(cards6-[a-z0-9-]+)\.texts\.tsv$", src)
        if not m:
            continue
        log = subprocess.run(["git", "-C", ROOT, "log", "--oneline", "--grep", f"^{m.group(1)}:",
                              *refs], capture_output=True, text=True).stdout.strip()
        if log:
            print(f"  {m.group(1)}: {len(log.splitlines())} commits")
    for p in problems:
        print("PROBLEM", p)
    if problems:
        print(f"FAILED: {len(problems)} problems")
        sys.exit(1)
    print("OK: every live unsupported text is placed exactly once")


if __name__ == "__main__":
    main()
