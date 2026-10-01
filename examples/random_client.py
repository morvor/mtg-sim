#!/usr/bin/env python3
"""A minimal mtg-sim agent: plays by choosing random legal options.

It speaks the protocol of docs/AGENT_PROTOCOL.md over stdin/stdout: each line it reads
is a JSON message ("decision" or "game_over"); for each decision it writes one line
with a JSON answer. It uses only the Python standard library.

    cargo run --release -p mtg-sim -- --games 1 \\
        --agent 0=random --agent 1=cmd:'python3 examples/random_client.py'

Options (environment variables):
    MTG_CLIENT_SEED   random seed (default: random)
    MTG_CLIENT_LOG    file to append a readable log of decisions and events to
"""

import json
import os
import random
import sys

rng = random.Random(os.environ.get("MTG_CLIENT_SEED"))
log_path = os.environ.get("MTG_CLIENT_LOG")
log_file = open(log_path, "a") if log_path else None


def log(text):
    if log_file:
        log_file.write(text + "\n")
        log_file.flush()


def choose_many(spec, options):
    """Between min and max distinct options, at most `group_max` of each group, within
    the pawprint budget if any."""
    lo, hi = spec["min"], min(spec["max"], len(options) if spec["distinct"] else spec["max"])
    want = rng.randint(lo, max(lo, hi))
    order = list(range(len(options)))
    rng.shuffle(order)
    chosen, groups, spent = [], {}, 0
    budget = spec.get("budget")
    if not spec["distinct"]:
        order = [rng.randrange(len(options)) for _ in range(want)] if options else []
    for i in order:
        if len(chosen) >= want:
            break
        opt = options[i]
        group = opt.get("group")
        if group is not None and "group_max" in opt and groups.get(group, 0) >= opt["group_max"]:
            continue
        cost = opt.get("cost", 0)
        if budget is not None and spent + cost > budget:
            continue
        chosen.append(i)
        if group is not None:
            groups[group] = groups.get(group, 0) + 1
        spent += cost
    return chosen


def divide(total, n, min_each):
    """`n` random numbers, each at least `min_each`, adding up to `total`."""
    parts = [min_each] * n
    for _ in range(total - min_each * n):
        parts[rng.randrange(n)] += 1
    return parts


def assign_damage(spec, options):
    """Random legal combat damage: lethal damage to the creatures in order while it
    lasts (always legal with trample), then the rest to one random recipient that may
    take it."""
    total, n = spec["total"], len(options)
    amounts = [0] * n
    left = total
    if not spec["trample"]:
        return divide(total, n, 0)
    # Recipients with `lethal` come first; trample lets the rest go past them only once
    # each has lethal damage.
    for i, opt in enumerate(options):
        need = opt.get("lethal")
        if need is None:
            break
        give = min(need, left)
        amounts[i] += give
        left -= give
    if left > 0:
        # Everything that got lethal (or the first non-lethal-tracked recipient) may take
        # more; put the excess on the last recipient that has met its requirement.
        candidates = [i for i in range(n)
                      if options[i].get("lethal") is None or amounts[i] >= options[i]["lethal"]]
        target = candidates[-1] if candidates else n - 1
        if rng.random() < 0.5 and candidates:
            target = rng.choice(candidates)
            # Damage to the planeswalker's controller needs the planeswalker's lethal
            # first, and anything past the blockers needs all blockers lethal: the
            # sequential fill above guarantees both whenever `left` > 0.
        amounts[target] += left
    return amounts


def answer(req):
    spec, options = req["answer"], req["options"]
    kind = spec["type"]
    if kind == "choose_one":
        if req["kind"] == "priority":
            # Rarely concede, prefer doing something over passing.
            choices = [o for o in options if o["action"]["type"] != "concede"]
            act = [o for o in choices if o["action"]["type"] not in ("pass", "mana_ability")]
            if act and rng.random() < 0.7:
                return {"index": rng.choice(act)["index"]}
            return {"index": 0}  # option 0 is always "pass"
        return {"index": rng.randrange(len(options))}
    if kind == "yes_no":
        return {"bool": rng.random() < 0.5 if req["kind"] != "mulligan" else False}
    if kind == "choose_many":
        return {"indices": choose_many(spec, options)}
    if kind == "order":
        order = list(range(len(options)))
        rng.shuffle(order)
        return {"indices": order}
    if kind == "number":
        lo = spec["min"]
        hi = spec.get("max")
        if hi is None:
            hi = lo + 2
        return {"number": rng.randint(lo, max(lo, min(hi, lo + 10)))}
    if kind == "divide":
        return {"numbers": divide(spec["total"], len(options), spec["min_each"])}
    if kind == "assign_damage":
        return {"numbers": assign_damage(spec, options)}
    if kind == "split":
        idx = list(range(len(options)))
        rng.shuffle(idx)
        k = rng.randint(0, len(idx))
        return {"split": [idx[:k], idx[k:]]}
    if kind == "text":
        # Name a card we can see, or a well-known one.
        names = [o.get("name") for o in req.get("observation", {}).get("battlefield", [])
                 if o.get("name")]
        return {"text": rng.choice(names) if names else "Lightning Bolt"}
    return {"default": True}


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        msg = json.loads(line)
        if msg.get("type") == "game_over":
            log("game over: %s" % json.dumps(msg.get("result")))
            break
        for ev in msg.get("events", []):
            log("  " + ev["text"])
        if msg.get("error"):
            log("  rejected: " + msg["error"])
        reply = answer(msg)
        reply["id"] = msg["id"]
        log("%s: %s -> %s" % (msg["kind"], msg["prompt"], json.dumps(reply)))
        sys.stdout.write(json.dumps(reply) + "\n")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
