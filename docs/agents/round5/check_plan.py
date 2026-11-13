#!/usr/bin/env python3
"""Check the round-5 card-support plan: every unsupported text is placed exactly once.

The input snapshot is round5/baseline.tsv (`mtg-tools unsupported --paper --by-card` when the
plan was made: one card<TAB>text line per unsupported text of a paper card, numbers as N, the
card's own name as ~; since paper-scope-1 the tools' `--paper` uses the paper rule of
`ScryfallCard::is_paper`). Every distinct text in it must appear in exactly one of:

- a round-5 item list, items/cards5-*.texts.tsv (including cards5-manual-defs);
- round5/inflight.tsv (owned by a round-4 item still in progress; each row must also be in
  that item's .texts.tsv);
- round5/pending-recheck.tsv (its first failing construct belongs to an in-flight item's family;
  re-probe it when that item lands);
- round5/deferred.tsv (out of scope under docs/DEFERRED.md; rows whose category starts with
  "digital version" are digital versions the paper rule leaves out, recorded but not counted);

and those files must not list texts that aren't in the snapshot. The card counts and example
cards in the lists must match the snapshot. The item table of card-support-plan-5.md must match
the lists (texts, paper cards touched, cards unlocked alone before the ALSO BLOCKED BY
exclusions, which the table states separately). Exit status 1 on any problem.

  check_plan.py           # check the plan files against the snapshot
  check_plan.py --live    # also compare the snapshot with the current release mtg-tools
                          # (target/release/mtg-tools of this checkout): texts supported since are
                          # reported, a live text that no list places is a problem; and say which
                          # in-flight items have landed since, in this checkout or on the
                          # fetched integration branch (their rows then need re-planning)
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


def run_tools(*args):
    tools = os.path.join(ROOT, "target", "release", "mtg-tools")
    out = subprocess.run([tools, "unsupported", *args, "--by-card"], capture_output=True,
                         text=True, check=True).stdout
    return {tuple(l.split("\t", 1)) for l in out.splitlines() if "\t" in l}


def plan_table():
    """{item: (texts, touched, alone)} from the work-item table of card-support-plan-5.md."""
    path = os.path.join(AGENTS, "card-support-plan-5.md")
    out = {}
    if not os.path.exists(path):
        return out
    for line in open(path, encoding="utf-8"):
        m = re.match(r"^\|\s*\d+\s*\|\s*(cards5-[a-z0-9-]+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)", line)
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
    for p in sorted(glob.glob(os.path.join(AGENTS, "items", "cards5-*.texts.tsv"))):
        sources[os.path.relpath(p, AGENTS)] = read_tsv(p)
        if not os.path.exists(p[:-len(".texts.tsv")] + ".md"):
            problems.append(f"{os.path.relpath(p, AGENTS)}: no brief next to it")
    for name in ("inflight.tsv", "pending-recheck.tsv", "deferred.tsv"):
        sources["round5/" + name] = read_tsv(os.path.join(HERE, name))
    seen = collections.defaultdict(list)
    for src, rows in sources.items():
        for r in rows:
            t = r["text"]
            if src == "round5/deferred.tsv" and r.get("category", "").startswith("digital version"):
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
    for name, col in (("inflight.tsv", "item"), ("pending-recheck.tsv", "pending_item")):
        for r in sources["round5/" + name]:
            path = os.path.join(AGENTS, "items", r[col] + ".texts.tsv")
            if not os.path.exists(path):
                problems.append(f"round5/{name}: no such item list: {r[col]}")
            elif name == "inflight.tsv" and r["text"] not in {x["text"] for x in read_tsv(path)}:
                problems.append(f"round5/inflight.tsv: not in {r[col]}'s list: {r['text'][:80]}")
    for t in cards_of:
        if t not in seen:
            problems.append(f"unplaced: {t[:100]} [{sorted(cards_of[t])[0]}]")
        elif len(seen[t]) > 1:
            problems.append(f"placed {len(seen[t])} times ({', '.join(seen[t])}): {t[:80]}")
    # Per-list counts, and the plan's item table.
    card_texts = collections.defaultdict(set)
    for r in base:
        card_texts[r["card"]].add(r["text"])
    owner = {t: s[0] for t, s in seen.items() if t in cards_of}
    print(f"snapshot: {len(base)} card-text lines, {len(cards_of)} distinct texts, "
          f"{len(card_texts)} cards")
    print("  texts  touched  alone  list")
    table = plan_table()
    groups = dict((s, {t for t, o in owner.items() if o == s}) for s in sources)
    for name, rows in (("inflight.tsv", "item"), ("pending-recheck.tsv", "pending_item")):
        for r in sources["round5/" + name]:
            groups.setdefault(f"round5/{name}:{r[rows]}", set()).add(r["text"])
    for src in sorted(groups):
        ts = groups[src]
        touched = {c for c, tt in card_texts.items() if tt & ts}
        alone = {c for c, tt in card_texts.items() if tt <= ts}
        print(f"  {len(ts):5d}  {len(touched):7d}  {len(alone):5d}  {src}")
        m = re.match(r"items/(cards5-[a-z0-9-]+)\.texts\.tsv$", src)
        if m and table:
            got = (len(ts), len(touched), len(alone))
            if m.group(1) not in table:
                problems.append(f"card-support-plan-5.md: no table row for {m.group(1)}")
            elif table[m.group(1)] != got:
                problems.append(f"card-support-plan-5.md: {m.group(1)} row says "
                                f"{table[m.group(1)]}, the list gives {got} (texts, touched, alone)")
    if "--live" in sys.argv:
        live = run_tools("--paper")
        snap = {(r["card"], r["text"]) for r in base}
        print(f"live: {len(live)} card-text lines; supported since the snapshot: "
              f"{len(snap - live)}; new since: {len(live - snap)}")
        for c, t in sorted(snap - live):
            print(f"  SUPPORTED {c}: {t[:90]} [{owner.get(t, '?')}]")
        for c, t in sorted(live - snap):
            print(f"  NEW {c}: {t[:100]}")
            if t not in seen:
                problems.append(f"live text placed nowhere: {t[:100]} [{c}]")
        items = sorted({r["item"] for r in sources["round5/inflight.tsv"]} |
                       {r["pending_item"] for r in sources["round5/pending-recheck.tsv"]})
        for it in items:
            # Look at this checkout and at the integration branch as last fetched: an item can
            # land there before this checkout merges it.
            refs = ["HEAD"] + [r for r in (INTEGRATION,) if subprocess.run(
                ["git", "-C", ROOT, "rev-parse", "--verify", "-q", r],
                capture_output=True).returncode == 0]
            log = subprocess.run(["git", "-C", ROOT, "log", "--oneline", "--grep", f"^{it}:",
                                  *refs], capture_output=True, text=True).stdout.strip()
            state = f"has landed ({len(log.splitlines())} commits): re-plan its rows" if log \
                else "still in flight"
            print(f"  {it}: {state}")
    for p in problems:
        print("PROBLEM", p)
    if problems:
        print(f"FAILED: {len(problems)} problems")
        sys.exit(1)
    print("OK: every unsupported text is placed exactly once")


if __name__ == "__main__":
    main()
