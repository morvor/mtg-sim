#!/usr/bin/env python3
"""Work plan for testing every Scryfall ruling.

Coverage is counted per ruling text: a text shared by several cards is covered by a test
on any card it applies to (choose a card with typical wording; when the cards word the
relevant ability differently, test each wording). Whether every individual card compiles
and behaves correctly is checked separately, on every card.

This script splits every ruling text into batches of similar ones, deterministically from
the data files alone, so a batch id (P001, ...) means the same thing on every machine.
Texts are grouped by the keyword they discuss (Scryfall `keywords`) or, otherwise, by the
cards' function (Scryfall Tagger oracle tags), so a batch covers one kind of ability.

Status comes from the repo: a text is CITED when a test has ruling!("<card or face name>",
"<substring of that ruling>") for any card with that ruling, EXEMPT when
docs/rulings-exemptions/*.tsv lists it as having no engine-testable content, DEFERRED when
every card it applies to is deferred (docs/DEFERRED.md: digital-only cards, sticker sheets),
OPEN otherwise.

Usage (from the repo root):
  python3 scripts/rulings_batches.py summary            # overall coverage
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
BUDGET = 150  # ruling texts per batch
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


def deferred(c):
    """Cards the project has decided not to support for now (docs/DEFERRED.md)."""
    games = c.get('games') or []
    if games and 'paper' not in games:
        return 'digital-only'
    if 'Stickers' in (c.get('type_line') or ''):
        return 'sticker sheet'
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
    batches = [groups[i:i + BUDGET] for i in range(0, len(groups), BUDGET)]
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


def status_fn(names, cards):
    cites = citations(names)
    ex = exemptions(names)

    def status(oids, text):
        n = normalize(text)
        if any(c in n for o in oids for c in cites.get(o, ())):
            return 'CITED'
        if any(e in n for e in ex):
            return 'EXEMPT'
        if all(deferred(cards[o]) for o in oids):
            return 'DEFERRED'
        return 'OPEN'
    return status


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return
    cards, names, batches, oos = plan()
    status = status_fn(names, cards)
    cmd = args[0]
    if cmd == 'summary':
        c = collections.Counter(status(os_, t) for b in batches.values() for _, t, os_ in b)
        tot = sum(c.values())
        print(f'ruling texts {tot} in {len(batches)} batches: CITED {c["CITED"]} ({100 * c["CITED"] / tot:.1f}%), '
              f'EXEMPT {c["EXEMPT"]}, OPEN {c["OPEN"]}, DEFERRED (digital-only cards, sticker sheets) {c["DEFERRED"]}; '
              f'out of scope (Contraptions, host/augment): {len(oos)}')
    elif cmd == 'list':
        for bid, b in batches.items():
            c = collections.Counter(status(os_, t) for _, t, os_ in b)
            keys = sorted({k for k, _, _ in b})
            print(f'{bid}\ttexts {len(b)}\topen {c["OPEN"]}\t{"; ".join(keys)[:150]}')
    elif cmd == 'show':
        bid = args[1]
        only_open = '--open' in args
        out = []
        for key, t, os_ in batches[bid]:
            st = status(os_, t)
            if only_open and st != 'OPEN':
                continue
            names_ = [cards[o]['name'] for o in os_]
            more = f' (+{len(names_) - 60} more)' if len(names_) > 60 else ''
            out.append({'group': key, 'ruling': t, 'status': st,
                        'cards': names_[:60] + ([more.strip()] if more else [])})
        print(json.dumps(out, indent=1, ensure_ascii=False))
    elif cmd == 'card':
        oid = names.get(args[1].lower())
        if not oid:
            sys.exit(f'unknown card {args[1]!r}')
        for bid, b in batches.items():
            for _, t, os_ in b:
                if oid in os_:
                    print(f'{status(os_, t):6} {bid}  {t}')
    else:
        sys.exit(__doc__)


if __name__ == '__main__':
    main()
