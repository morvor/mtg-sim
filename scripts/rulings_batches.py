#!/usr/bin/env python3
"""Work plan for testing every Scryfall ruling on every card it applies to.

Each (card, ruling) pair needs its own test on that card: a ruling cited on one card
does not cover another card with the same ruling text, because cards worded slightly
differently can behave differently. This script splits every pair into batches of
similar cards, deterministically from the data files alone, so a batch id (P001, ...)
means the same thing on every machine:

  - pairs whose ruling text is shared by several cards stay together (one test can run
    the same scenario over each card, with a ruling!() citation per card);
  - texts are grouped by the keyword they discuss (Scryfall `keywords`) or, otherwise,
    by the cards' function (Scryfall Tagger oracle tags), so a batch covers one kind of
    ability;
  - batches are packed to a similar amount of work (an extra card for a text already
    being tested costs less than a new text).

Status comes from the repo: a pair is CITED when a test has ruling!("<card or face
name>", "<substring of that ruling>"), EXEMPT when docs/rulings-exemptions/*.tsv lists
the text as having no engine-testable content, OPEN otherwise.

Usage (from the repo root):
  python3 scripts/rulings_batches.py summary            # overall pair coverage
  python3 scripts/rulings_batches.py list               # every batch with status counts
  python3 scripts/rulings_batches.py show P042 [--open] # one batch as JSON
  python3 scripts/rulings_batches.py card "Card Name"   # one card's rulings and status
"""
import collections
import glob
import gzip
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUDGET = 150.0  # work units per batch: 1 per text plus EXTRA per additional card
EXTRA = 0.35
EXCLUDED_LAYOUTS = {'art_series', 'token', 'double_faced_token', 'emblem'}
# Tagger tags about names, art, flavor or printing say nothing about what a card does.
NONFUNCTIONAL = re.compile(
    r'portmanteau|pun|alliteration|english word|fun ruling|black border|aesthetic|'
    r'flavor|art-|-art|reprint|misprint|storyline|in-cards|cycle|name|reference|joke|'
    r'homage|tribute|iconic|sketch', re.I)


def normalize(s):
    s = (s.replace('‘', "'").replace('’', "'").replace('‛', "'")
         .replace('“', '"').replace('”', '"')
         .replace('–', '-').replace('—', '-').replace('−', '-'))
    return ' '.join(s.split())


def faces(c):
    return c.get('card_faces') or [c]


def out_of_scope(c):
    """Contraptions and host/augment cards are out of scope for now."""
    if c.get('layout') in ('host', 'augment'):
        return 'host/augment'
    txt = ' '.join((f.get('type_line') or '') + ' ' + (f.get('oracle_text') or '')
                   for f in faces(c))
    if 'Contraption' in txt or re.search(r'\bassemble\b', txt, re.I):
        return 'Contraptions'
    return None


def load():
    cards = {}  # oracle_id -> card
    for line in gzip.open(os.path.join(ROOT, 'data/oracle_cards.jsonl.gz'), 'rt'):
        c = json.loads(line)
        oid = c.get('oracle_id')
        if oid and oid not in cards and c.get('layout') not in EXCLUDED_LAYOUTS:
            cards[oid] = c
    names = {}  # lowercase full or face name -> oracle_id (like CardDb::by_name)
    for oid in sorted(cards, key=lambda o: cards[o]['name']):
        c = cards[oid]
        for n in [c['name']] + [f.get('name', '') for f in faces(c)]:
            if n:
                names.setdefault(n.lower(), oid)
    texts = collections.defaultdict(set)  # oracle_id -> ruling texts
    for line in gzip.open(os.path.join(ROOT, 'data/rulings.jsonl.gz'), 'rt'):
        r = json.loads(line)
        if r.get('oracle_id') in cards:
            texts[r['oracle_id']].add(r['comment'])
    tags = collections.defaultdict(list)  # oracle_id -> [(tag, global size)]
    for line in gzip.open(os.path.join(ROOT, 'data/oracle_tags.jsonl.gz'), 'rt'):
        t = json.loads(line)
        if t.get('type') != 'oracle' or NONFUNCTIONAL.search(t['label']):
            continue
        ids = sorted({x['oracle_id'] for x in t.get('taggings', [])})
        for o in ids:
            if o in cards:
                tags[o].append((t['label'], len(ids)))
    return cards, names, texts, tags


def function_tag(tagl):
    """A medium-specific functional tag: the broadest tag with 20-400 cards, else the
    tag closest to that range."""
    if not tagl:
        return None
    inside = [t for t in tagl if 20 <= t[1] <= 400]
    if inside:
        return max(inside, key=lambda t: (t[1], t[0]))[0]
    return min(tagl, key=lambda t: (min(abs(t[1] - 20), abs(t[1] - 400)), t[0]))[0]


def type_key(c):
    tl = (c.get('type_line') or faces(c)[0].get('type_line') or '').split(' // ')[0]
    for k in ['Creature', 'Planeswalker', 'Instant', 'Sorcery', 'Enchantment', 'Artifact',
              'Land', 'Battle']:
        if k in tl:
            return 'type: ' + k.lower()
    return 'type: other'


def plan():
    cards, names, texts, tags = load()
    by_text = collections.defaultdict(list)
    oos = []
    for oid in sorted(texts, key=lambda o: cards[o]['name']):
        why = out_of_scope(cards[oid])
        for t in sorted(texts[oid]):
            if why:
                oos.append((cards[oid]['name'], t, why))
            else:
                by_text[t].append(oid)
    groups = []  # (key, text, [oid])
    for t in sorted(by_text):
        oids = by_text[t]
        low = t.lower()
        shared_kw = sorted(set.intersection(*[set(cards[o].get('keywords') or []) for o in oids]))
        kw = [k for k in shared_kw if k.lower() in low]
        if kw:
            key = 'keyword: ' + kw[0]
        else:
            ft = collections.Counter(function_tag(tags[o]) for o in oids)
            ft.pop(None, None)
            if ft:
                key = 'function: ' + min(ft.items(), key=lambda kv: (-kv[1], kv[0]))[0]
            else:
                key = type_key(cards[oids[0]])
        groups.append((key, t, oids))
    groups.sort(key=lambda g: (g[0], g[1]))
    batches = []
    cur, cur_w = [], 0.0
    for key, t, oids in groups:
        chunk = []
        for o in oids:
            w = 1.0 if not chunk else EXTRA
            if cur_w + w > BUDGET and (cur or chunk):
                if chunk:
                    cur.append((key, t, chunk))
                batches.append(cur)
                cur, cur_w, chunk = [], 0.0, []
                w = 1.0
            chunk.append(o)
            cur_w += w
        if chunk:
            cur.append((key, t, chunk))
    if cur:
        batches.append(cur)
    ids = {f'P{i:03d}': b for i, b in enumerate(batches, 1)}
    return cards, names, ids, oos


def citations(names):
    """(oracle_id, normalized needle) pairs cited anywhere in the workspace sources."""
    pat = re.compile(r'ruling!\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"', re.S)
    out = collections.defaultdict(list)
    for f in glob.glob(os.path.join(ROOT, 'crates/**/*.rs'), recursive=True):
        src = open(f, encoding='utf-8').read()
        if 'ruling!' not in src:
            continue
        for m in pat.finditer(src):
            oid = names.get(m.group(1).replace('\\"', '"').lower())
            if oid:
                needle = m.group(2).replace('\\"', '"').replace('\\\\', '\\').replace('\\n', ' ')
                out[oid].append(normalize(needle))
    return out


def exemptions(names):
    out = collections.defaultdict(list)  # needle -> reasons, applied to every card
    for f in sorted(glob.glob(os.path.join(ROOT, 'docs/rulings-exemptions/*.tsv'))):
        for line in open(f, encoding='utf-8'):
            if line.startswith('#') or not line.strip():
                continue
            p = line.rstrip('\n').split('\t')
            if len(p) >= 2:
                out[normalize(p[1])].append(p[2] if len(p) > 2 else '')
    return out


def status_fn(names):
    cites = citations(names)
    ex = exemptions(names)

    def status(oid, text):
        n = normalize(text)
        if any(c in n for c in cites.get(oid, ())):
            return 'CITED'
        if any(e in n for e in ex):
            return 'EXEMPT'
        return 'OPEN'
    return status


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return
    cards, names, batches, oos = plan()
    status = status_fn(names)
    cmd = args[0]
    if cmd == 'summary':
        c = collections.Counter(status(o, t) for b in batches.values() for _, t, os_ in b for o in os_)
        tot = sum(c.values())
        print(f'pairs {tot} in {len(batches)} batches: CITED {c["CITED"]} ({100 * c["CITED"] / tot:.1f}%), '
              f'EXEMPT {c["EXEMPT"]}, OPEN {c["OPEN"]}; out of scope (Contraptions, host/augment): {len(oos)}')
    elif cmd == 'list':
        for bid, b in batches.items():
            c = collections.Counter(status(o, t) for _, t, os_ in b for o in os_)
            keys = sorted({k for k, _, _ in b})
            print(f'{bid}\tpairs {sum(c.values())}\topen {c["OPEN"]}\ttexts {len(b)}\t{"; ".join(keys)[:150]}')
    elif cmd == 'show':
        bid = args[1]
        only_open = '--open' in args
        out = []
        for key, t, os_ in batches[bid]:
            cs = [{'card': cards[o]['name'], 'status': status(o, t)} for o in os_]
            if only_open:
                cs = [x for x in cs if x['status'] == 'OPEN']
            if cs:
                out.append({'group': key, 'ruling': t, 'cards': cs})
        print(json.dumps(out, indent=1, ensure_ascii=False))
    elif cmd == 'card':
        oid = names.get(args[1].lower())
        if not oid:
            sys.exit(f'unknown card {args[1]!r}')
        where = {(o, t): bid for bid, b in batches.items() for _, t, os_ in b for o in os_}
        for (o, t), bid in sorted(where.items(), key=lambda kv: kv[0][1]):
            if o == oid:
                print(f'{status(o, t):6} {bid}  {t}')
    else:
        sys.exit(__doc__)


if __name__ == '__main__':
    main()
