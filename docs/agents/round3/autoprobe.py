#!/usr/bin/env python3
"""Find the failing piece of unsupported texts by compiler probes (round-3 planning).

Input: a JSONL file with one object per text: {"text", "cards", "example", "raw"} (`raw`
is the example card's unsupported ability as `mtg-tools unsupported --by-card --raw`
prints it: the card's name as ~, lines joined by " / "). For every text it probes, in one
run of the compiler (probe.py), with the card's other paragraphs kept as context:

- the cost of an activated ability ("COST: Draw a card.") and its effect ("{T}: EFFECT");
- the trigger condition of a triggered ability (the longest "TRIGGER, draw a card." that
  compiles, so an intervening if that compiles stays with the trigger);
- the effect sentences added one by one (first failing prefix = failing sentence);
- for modal abilities, the header with plain modes and each mode on its own.

Output: JSONL with the input fields plus "piece_kind" (cost, trigger, header, mode,
sentence, combination, whole, error) and "failing_piece".
"""
import difflib
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe  # noqa: E402

FILLERS = ["Draw a card.", "You gain 1 life.", "Scry 1.", "Each opponent loses 1 life.",
           "Create a Treasure token.", "Mill a card.", "Surveil 1.", "Investigate."]


def split_sentences(s):
    out, cur, depth = [], "", False
    i = 0
    while i < len(s):
        ch = s[i]
        cur += ch
        if ch == '"':
            depth = not depth
        if ch == "." and not depth and (i + 1 == len(s) or s[i + 1] == " "):
            out.append(cur.strip())
            cur = ""
        i += 1
    if cur.strip():
        out.append(cur.strip())
    return out


def top_level_commas(s):
    pos, q = [], False
    for i, ch in enumerate(s):
        if ch == '"':
            q = not q
        elif ch == "," and not q and s[i + 1:i + 2] == " ":
            pos.append(i)
    return pos


COST_START = re.compile(r"^(\{|[+\-−]?[0-9X]+$|[+\-−][0-9X]|Sacrifice|Exile|Tap|Untap|Pay|Discard|Remove|Put|Return|Reveal|Mill|Collect|Forage|Waterbend|Earthbend|Blight)")


def split_label(line):
    """'Ability word — rest' -> (label prefix, rest)."""
    m = re.match(r"^([A-Z][^—:{}\"]{0,40}? — )(.*)$", line)
    if m and not m.group(1).startswith(("Choose", "choose")):
        return m.group(1), m.group(2)
    return "", line


def activated_split(line):
    q = False
    for i, ch in enumerate(line):
        if ch == '"':
            q = not q
        if ch == ":" and not q and line[i + 1:i + 2] == " ":
            cost = line[:i]
            if COST_START.match(cost.strip()) and len(cost) < 120:
                return cost, line[i + 2:]
            return None
    return None


def other_paragraphs(card, raw):
    """The card face's other paragraphs (name as ~), and the face index."""
    names = probe.self_names(card)
    faces = card.get("card_faces") or [card]
    target = raw.replace(" / ", "\n")
    best = (-1.0, 0, 0)
    face_pars = []
    for fi, fc in enumerate(faces):
        ot = fc.get("oracle_text", "") or ""
        for n in names:
            ot = ot.replace(n, "~")
        ot = re.sub(r" ?\([^()]*\)", "", ot)
        pars = [p for p in ot.split("\n") if p.strip()]
        face_pars.append(pars)
    # Paragraph groups: a multi-line ability spans several consecutive paragraphs.
    n_lines = target.count("\n") + 1
    for fi, pars in enumerate(face_pars):
        for pi in range(len(pars)):
            chunk = "\n".join(pars[pi:pi + n_lines])
            r = difflib.SequenceMatcher(None, chunk, target).ratio()
            if r > best[0]:
                best = (r, fi, pi)
    _, fi, pi = best
    pars = face_pars[fi] if face_pars else []
    before = pars[:pi]
    after = pars[pi + n_lines:]
    return before, after, fi


def variants_for(raw):
    """[(kind, piece, variant_ability_text)] in probing order."""
    lines = raw.split(" / ")
    out = []
    bullets = [i for i, l in enumerate(lines) if l.startswith("•") or l.startswith("+ ")]
    if bullets and len(lines) > 1:
        header = lines[:bullets[0]]
        modes = [lines[i] for i in bullets]
        rest = lines[bullets[-1] + 1:]
        bullet_mark = lambda l: "• " if l.startswith("•") else ""
        plain = []
        for k, m in enumerate(modes):
            if m.startswith("+ "):
                cost = m.split(" — ", 1)[0]
                plain.append(f"{cost} — {FILLERS[k % len(FILLERS)]}")
            else:
                plain.append("• " + FILLERS[k % len(FILLERS)])
        out.append(("header", " / ".join(header) or "(modal)", " / ".join(header + plain + rest)))
        for k, m in enumerate(modes):
            v = plain[:k] + [m] + plain[k + 1:]
            out.append(("mode", m, " / ".join(header + v + rest)))
        return out
    if len(lines) > 1:
        # Level up, class levels, multi-line abilities: probe each line in place.
        for k, l in enumerate(lines):
            out.append(("line", l, " / ".join(lines[:k + 1])))
        return out
    line = lines[0]
    label, body = split_label(line)
    act = activated_split(body)
    if act:
        cost, eff = act
        loyalty = re.match(r"^[+\-−]?[0-9X]+$", cost.strip())
        out.append(("cost", cost, f"{label}{cost}: Draw a card."))
        base = cost if loyalty else "{T}"
        sents = split_sentences(eff)
        for j in range(1, len(sents) + 1):
            out.append(("sentence", sents[j - 1], f"{label}{base}: " + " ".join(sents[:j])))
        return out
    if re.match(r"^(When|Whenever|At)\b", body):
        sents = split_sentences(body)
        first = sents[0]
        commas = top_level_commas(first)
        trig_variants = []
        for c in commas:
            trig_variants.append(("trigger", first[:c], f"{label}{first[:c]}, draw a card."))
        out.extend(trig_variants)
        # Effect sentences, with the split chosen later (see resolve()).
        out.append(("__effect__", body, label))
        return out
    sents = split_sentences(body)
    for j in range(1, len(sents) + 1):
        out.append(("sentence", sents[j - 1], label + " ".join(sents[:j])))
    return out


def context_text(before, after, ability):
    return "\n".join(before + [ability.replace(" / ", "\n")] + after)


def run_batch(items):
    """items: list of (card, text, face). Returns list of fail lists."""
    probes = [{"card": c, "text": t, "face": f} for c, t, f in items]
    res = []
    for k in range(0, len(probes), 3000):
        res.extend(probe.run(probes[k:k + 3000], TOOLS))
    return res


def norm(s):
    return re.sub(r"\s+", " ", s.replace(" / ", " ")).strip().lower()


def main():
    inp, outp = sys.argv[1], sys.argv[2]
    texts = [json.loads(l) for l in open(inp) if l.strip()]
    plans = []
    batch = []
    for t in texts:
        card = probe.cards().get(t["example"].lower())
        if card is None:
            plans.append(None)
            continue
        before, after, fi = other_paragraphs(card, t["raw"])
        face = fi if card.get("card_faces") else None
        vs = variants_for(t["raw"])
        p = {"before": before, "after": after, "face": face, "vs": vs, "idx": []}
        # Baseline: the unchanged text, to learn which failures are context.
        p["base"] = len(batch)
        batch.append((t["example"], context_text(before, after, t["raw"]), face))
        for kind, piece, v in vs:
            if kind == "__effect__":
                continue
            p["idx"].append(len(batch))
            batch.append((t["example"], context_text(before, after, v), face))
        plans.append(p)
    res = run_batch(batch)

    def bad(p, fails, ability_variant):
        base_fails = res[p["base"]]
        ctx = {norm(x) for x in base_fails} - {norm(TEXTS_RAW[id(p)])}
        return [x for x in fails if norm(x) not in ctx]

    TEXTS_RAW = {}
    # Second pass: trigger effect sentences once the trigger split is known.
    batch2 = []
    for t, p in zip(texts, plans):
        if p is None:
            continue
        TEXTS_RAW[id(p)] = t["raw"]
        eff = [v for v in p["vs"] if v[0] == "__effect__"]
        if not eff:
            continue
        _, body, label = eff[0]
        trig_ok = None
        k = 0
        for kind, piece, v in p["vs"]:
            if kind == "__effect__":
                continue
            if kind == "trigger" and not bad(p, res[p["idx"][k]], v):
                trig_ok = piece  # keep the longest that compiles
            k += 1
        p["trig_ok"] = trig_ok
        if trig_ok is None:
            continue
        rest = body[len(trig_ok) + 2:]
        sents = split_sentences(rest)
        p["eff_idx"] = []
        for j in range(1, len(sents) + 1):
            v = f"{label}{trig_ok}, " + " ".join(sents[:j])
            p["eff_idx"].append((sents[j - 1], len(batch2)))
            batch2.append((t["example"], context_text(p["before"], p["after"], v), p["face"]))
    res2 = run_batch(batch2) if batch2 else []

    with open(outp, "w") as out:
        for t, p in zip(texts, plans):
            r = dict(t)
            if p is None:
                r.update(piece_kind="error", failing_piece="example card not found")
                out.write(json.dumps(r) + "\n")
                continue
            base_bad = bad(p, res[p["base"]], t["raw"])
            if not base_bad:
                r.update(piece_kind="supported", failing_piece="")
                out.write(json.dumps(r) + "\n")
                continue
            kind_found = None
            k = 0
            first_fail = {}
            for kind, piece, v in p["vs"]:
                if kind == "__effect__":
                    continue
                failed = bool(bad(p, res[p["idx"][k]], v))
                k += 1
                if kind == "trigger":
                    continue
                if failed and kind not in first_fail:
                    first_fail[kind] = piece
                if failed and kind in ("header", "mode", "line") and kind_found is None:
                    kind_found = (kind, piece)
            if kind_found is None and "cost" in first_fail:
                kind_found = ("cost", first_fail["cost"])
            if kind_found is None and "sentence" in first_fail:
                kind_found = ("sentence", first_fail["sentence"])
            if kind_found is None and "__effect__" in [v[0] for v in p["vs"]]:
                if p.get("trig_ok") is None:
                    first = split_sentences(t["raw"])[0]
                    commas = top_level_commas(first)
                    kind_found = ("trigger", first[:commas[0]] if commas else first)
                else:
                    for s, j in p.get("eff_idx", []):
                        if bad(p, res2[j], s):
                            kind_found = ("sentence", s)
                            break
                    if kind_found is None:
                        kind_found = ("combination", t["raw"])
            if kind_found is None:
                kind_found = ("combination", t["raw"])
            r.update(piece_kind=kind_found[0], failing_piece=kind_found[1],
                     trigger_ok=p.get("trig_ok"))
            out.write(json.dumps(r) + "\n")


TOOLS = os.environ.get("MTG_TOOLS", probe.os.path.join(probe.ROOT, "target", "release", "mtg-tools"))

if __name__ == "__main__":
    main()
