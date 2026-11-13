#!/usr/bin/env python3
"""Check the round-4 card-support plan: every unsupported text is placed exactly once.

The input snapshot is round4/baseline.tsv (`mtg-tools unsupported --paper --by-card` when
the plan was made: one card<TAB>text line per unsupported text of a paper card, numbers as
N, the card's own name as ~). Every distinct text in it must appear in exactly one of:

- a round-4 item list, items/cards4-*.texts.tsv (including cards4-manual-defs);
- round4/inflight.tsv (owned by a round-3 item still in progress; each row must also be in that item's .texts.tsv);
- round4/pending-recheck.tsv (waiting for an in-progress item's construct);
- round4/deferred.tsv (out of scope under docs/DEFERRED.md);

and those files must not list texts that aren't in the snapshot. The card counts in the
lists must match the snapshot. Exit status 1 on any problem.

The snapshot also includes round4/paper-scope-gap.tsv (card, text, oracle set, formats): the
unsupported texts of paper cards that the tools' old `--paper` filter left out (their Scryfall
oracle record is an online-only reprint; items paper-scope-1 and cards4-paper-plan). Each of its
texts must be placed exactly once in the same lists; a text that is both in baseline.tsv and in
the gap list (Memory Jar shares Magus of the Jar's text) may carry either card count.

  check_plan.py           # check the plan files against the snapshot
  check_plan.py --live    # also compare the snapshot with the current release mtg-tools
                          # (texts supported since are reported; a live text that no list
                          # places is a problem, so the plan is checked against a fresh list)

With --live the paper set is computed twice: from `mtg-tools unsupported --paper --by-card`
and from `mtg-tools unsupported --by-card` filtered here by the paper rule of paper-scope-1
(`is_paper` in scripts/rulings_batches.py, the mirror of ScryfallCard::is_paper: games empty
or with "paper", or legal, restricted or banned in a paper format, digital versions aside). The rule's
list is the one checked, so the check means the same before and after paper-scope-1 lands; the
script says whether the tools' `--paper` list already agrees with the rule.
"""
import collections
import glob
import gzip
import json
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


sys.path.insert(0, os.path.join(ROOT, "scripts"))
from rulings_batches import is_paper  # noqa: E402  (paper-scope-1's rule, ScryfallCard::is_paper)


def paper_names_of(text):
    """The cards in the snapshot with this text (digital-version rows must have none)."""
    return {r["card"] for r in SNAPSHOT if r["text"] == text}


SNAPSHOT = []


def run_tools(*args):
    tools = os.path.join(ROOT, "target", "release", "mtg-tools")
    out = subprocess.run([tools, "unsupported", *args, "--by-card"], capture_output=True,
                         text=True, check=True).stdout
    return {tuple(l.split("\t", 1)) for l in out.splitlines() if "\t" in l}


def main():
    problems = []
    base = read_tsv(os.path.join(HERE, "baseline.tsv"))
    gap = read_tsv(os.path.join(HERE, "paper-scope-gap.tsv"))
    SNAPSHOT.extend(base + gap)
    cards_of = collections.defaultdict(set)
    for r in base:
        cards_of[r["text"]].add(r["card"])
    base_cards_of = {t: set(c) for t, c in cards_of.items()}
    for r in gap:
        cards_of[r["text"]].add(r["card"])
    gap_texts = {r["text"] for r in gap}
    sources = {}
    for p in sorted(glob.glob(os.path.join(AGENTS, "items", "cards4-*.texts.tsv"))):
        sources[os.path.relpath(p, AGENTS)] = read_tsv(p)
    for name in ("inflight.tsv", "pending-recheck.tsv", "deferred.tsv"):
        sources["round4/" + name] = read_tsv(os.path.join(HERE, name))
    seen = collections.defaultdict(list)
    for src, rows in sources.items():
        for r in rows:
            t = r["text"]
            if src == "round4/deferred.tsv" and r.get("category", "").startswith("digital version"):
                # A digital version the paper rule leaves out (DIGITAL_VERSIONS in
                # crates/mtg-data/src/scryfall.rs): recorded here, not a paper text.
                if r["example_card"] in paper_names_of(t):
                    problems.append(f"{src}: {r['example_card']} is a paper card: {t[:80]}")
                continue
            seen[t].append(src)
            if t not in cards_of:
                problems.append(f"{src}: text not in the snapshot: {t[:100]}")
                continue
            allowed = {len(cards_of[t]), len(base_cards_of.get(t, cards_of[t]))}
            if int(r["cards"]) not in allowed:
                problems.append(f"{src}: card count {r['cards']} != {len(cards_of[t])}: {t[:80]}")
            if r.get("example_card") and r["example_card"] not in cards_of[t]:
                problems.append(f"{src}: example card {r['example_card']} doesn't have: {t[:80]}")
    for name, col in (("inflight.tsv", "item"), ("pending-recheck.tsv", "pending_item")):
        for r in sources["round4/" + name]:
            path = os.path.join(AGENTS, "items", r[col] + ".texts.tsv")
            if not os.path.exists(path):
                problems.append(f"round4/{name}: no such item list: {r[col]}")
            elif name == "inflight.tsv" and r["text"] not in {x["text"] for x in read_tsv(path)}:
                problems.append(f"round4/inflight.tsv: not in {r[col]}'s list: {r['text'][:80]}")
    for t in cards_of:
        if t not in seen:
            problems.append(f"unplaced: {t[:100]} [{sorted(cards_of[t])[0]}]")
        elif len(seen[t]) > 1:
            problems.append(f"placed {len(seen[t])} times ({', '.join(seen[t])}): {t[:80]}")
    counts = collections.Counter(src for t in cards_of for src in seen.get(t, [])[:1])
    gap_counts = collections.Counter(src for t in gap_texts for src in seen.get(t, [])[:1])
    print(f"snapshot: {len(base)} + {len(gap)} card-text lines (baseline + paper-scope gap), "
          f"{len(cards_of)} distinct texts, {len({r['card'] for r in base + gap})} cards")
    print(f"paper-scope gap: {len(gap_texts)} texts, {len({r['card'] for r in gap})} cards "
          f"({len(gap_texts & set(base_cards_of))} text also in the baseline)")
    print("  texts   gap  list")
    for src in sorted(sources):
        print(f"  {counts[src]:5d}  {gap_counts[src]:5d}  {src}")
    if "--live" in sys.argv:
        paper_names = set()
        with gzip.open(os.path.join(ROOT, "data", "oracle_cards.jsonl.gz"), "rt",
                       encoding="utf-8") as f:
            for line in f:
                c = json.loads(line)
                # Like the tools: only playable records count (a token can share a card's
                # name: the Shapeshifter tokens and Antiquities' Shapeshifter).
                if c.get("layout") in ("art_series", "token", "double_faced_token", "emblem",
                                       "front_card"):
                    continue
                if is_paper(c):
                    paper_names.add(c["name"])
        tools_paper = run_tools("--paper")
        live = {(c, t) for c, t in run_tools() if c in paper_names}
        missing = live - tools_paper
        print(f"tools --paper: {len(tools_paper)} card-text lines; by the paper rule: {len(live)}"
              f" ({'the tools agree with the rule' if not missing and not tools_paper - live else f'{len(missing)} lines only by the rule, {len(tools_paper - live)} only in --paper: paper-scope-1 has not landed'})")
        snap = {(r["card"], r["text"]) for r in base + gap}
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
