#!/usr/bin/env python3
"""Probe the Oracle compiler with edited card texts (round-3 card-support planning).

Each probe is a real card whose text (one face) is replaced by a variant; the probes are
written as one-face cards into a temporary data directory and compiled in one run of the
release `mtg-tools unsupported --by-card --raw` (MTG_DATA_DIR). For every probe it prints
OK (everything compiles) or the paragraphs that still fail.

  probe.py --card "Oath of Druids" --text "At the beginning of each player's upkeep, ..."
  probe.py --file probes.jsonl      # one {"id": ..., "card": ..., "text": ...} per line

`text` is the whole face text to compile (paragraphs separated by newlines), with `~` for
the card's own name. `face` (optional, 0-based) picks the face of a multi-face card whose
types, cost and P/T the probe takes; by default the face whose text contains the first
paragraph of the probe text, else face 0. Options: --tools PATH (default
target/release/mtg-tools of this checkout), --json (machine-readable output).
"""
import argparse
import gzip
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
DATA = os.path.join(ROOT, "data", "oracle_cards.jsonl.gz")

_cards = None


def cards():
    global _cards
    if _cards is None:
        _cards = {}
        with gzip.open(DATA, "rt", encoding="utf-8") as f:
            for line in f:
                d = json.loads(line)
                _cards.setdefault(d["name"].lower(), d)
    return _cards


def self_names(card):
    names = [card["name"]]
    for face in card.get("card_faces") or []:
        names.append(face["name"])
    out = []
    for n in names:
        out.append(n)
        if "," in n:
            out.append(n.split(",")[0])
    return sorted(set(out), key=len, reverse=True)


def pick_face(card, text, face):
    faces = card.get("card_faces") or []
    if not faces:
        return card
    if face is not None:
        return faces[face]
    first = text.split("\n")[0].strip()[:40].replace("~", "")
    for fc in faces:
        ot = fc.get("oracle_text", "")
        for n in self_names(card):
            ot = ot.replace(n, "")
        if first and first.split(" ")[0] and first[:20] in ot:
            return fc
    return faces[0]


def make_probe(i, p):
    card = cards().get(p["card"].lower())
    if card is None:
        raise SystemExit(f"unknown card: {p['card']}")
    face = pick_face(card, p["text"], p.get("face"))
    name = f"Probe Card {chr(65 + i // 676)}{chr(65 + (i // 26) % 26)}{chr(65 + i % 26)}"
    d = dict(card)
    d.pop("card_faces", None)
    d["layout"] = "normal"
    d["name"] = name
    d["oracle_id"] = f"probe-{i}"
    d["id"] = f"probe-{i}"
    for k in ("mana_cost", "type_line", "power", "toughness", "loyalty", "defense"):
        if k in face:
            d[k] = face[k]
        elif face is not card:
            d.pop(k, None)
    d["oracle_text"] = p["text"].replace("~", name)
    return name, d


def run(probes, tools):
    with tempfile.TemporaryDirectory(prefix="probe-") as tmp:
        names = []
        with gzip.open(os.path.join(tmp, "oracle_cards.jsonl.gz"), "wt", encoding="utf-8") as f:
            for i, p in enumerate(probes):
                name, d = make_probe(i, p)
                names.append(name)
                f.write(json.dumps(d) + "\n")
        for extra in ("rulings.jsonl.gz", "oracle_tags.jsonl.gz", "comprehensive-rules.txt"):
            src = os.path.join(ROOT, "data", extra)
            if extra == "comprehensive-rules.txt":
                os.symlink(src, os.path.join(tmp, extra))
            else:
                with gzip.open(os.path.join(tmp, extra), "wt") as g:
                    g.write("")
        env = dict(os.environ, MTG_DATA_DIR=tmp)
        out = subprocess.run(
            [tools, "unsupported", "--by-card", "--raw"],
            env=env, capture_output=True, text=True,
        )
        if out.returncode != 0:
            raise SystemExit(f"mtg-tools failed: {out.stderr[-2000:]}")
        fails = {n: [] for n in names}
        for line in out.stdout.splitlines():
            if "\t" not in line:
                continue
            n, t = line.split("\t", 1)
            if n in fails:
                fails[n].append(t)
        return [fails[n] for n in names]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--card")
    ap.add_argument("--text")
    ap.add_argument("--face", type=int)
    ap.add_argument("--file")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--tools", default=os.path.join(ROOT, "target", "release", "mtg-tools"))
    a = ap.parse_args()
    if a.file:
        with open(a.file) as f:
            probes = [json.loads(l) for l in f if l.strip()]
    elif a.card and a.text is not None:
        probes = [{"id": "1", "card": a.card, "text": a.text, "face": a.face}]
    else:
        ap.error("--card and --text, or --file")
    results = run(probes, a.tools)
    for p, fails in zip(probes, results):
        if a.json:
            print(json.dumps({"id": p.get("id"), "ok": not fails, "fails": fails}))
        else:
            status = "OK" if not fails else "FAIL"
            print(f"[{p.get('id')}] {status}")
            for t in fails:
                print(f"    {t}")


if __name__ == "__main__":
    main()
