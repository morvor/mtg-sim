# rulings-residue: the open rulings left behind in batches already worked

WHY: docs/agents/finish-line-plan.md, phase A, item A3. 114 batches that earlier rounds worked
still hold 1,599 OPEN ruling texts between them (Nov 9; `python3 scripts/rulings_batches.py list`
shows each batch's OPEN count). Earlier rounds (the keyword sweeps, the rounds before "drive OPEN
to zero") left the harder or then-unsupported part; the 23 batches landed since Nov 5 left one
OPEN text in total. These leftovers are where the interaction bugs are (copies, counters,
reflexive triggers, type changes, forced attacks and blocks), so they are worked as themed
items, not batch by batch.

Each item is a rulings batch in every respect (the "Rulings batch" brief in
docs/agents/worker.md: cited / exempt / uncovered, the discrepancy policies, the report counts),
except that its open rulings come from several batches. Get them with
`python3 scripts/rulings_batches.py show PNNN --open` for each batch named; tests go in
`crates/mtg-engine/tests/rulings/r_residue_<topic>.rs`. Do the interaction rulings first. A
ruling left UNCOVERED for a named reason (the script has no such status; it stays OPEN) also
goes into docs/rulings-uncovered.tsv (`Card<TAB>substring<TAB>reason`), the list criterion C1
reconciles against the OPEN count. These items are the plan's stopping signal for the rulings
oracle (finish-line-plan §1.1), so the reviewer appends the item's row to docs/agents/yield.tsv
and reports how many of its rulings were interactions rather than restatements.

## Items (slug: batches; approximate OPEN count on Nov 9)

| Slug | Batches | OPEN | Topic |
|---|---|---|---|
| `rulings-residue-copy-creature` | P020 P021 P023 | 116 | creatures that copy, Clone exceptions, copies entering with choices |
| `rulings-residue-enchant-equip` | P207 P208 P209 P210 P211 P212 P213 | ~130 | Auras and Equipment keyword leftovers: attach legality, enchant restrictions, equip timing |
| `rulings-residue-counters` | P035 P119 P120 P121 | ~90 | counters: placement, removal, proliferate, doubling |
| `rulings-residue-reflexive-refund` | P129 P130 | ~65 | reflexive triggers and refunds |
| `rulings-residue-type-change` | P187 P188 | 58 | type-changing effects and layers |
| `rulings-residue-hate-attack-block` | P079 P080 P082 | 63 | attack and block requirements, restrictions, punishers |
| `rulings-residue-copy-instant` | P028 P030 P032 | 47 | copying spells on the stack |
| `rulings-residue-keywords` | P217 P214 P205 P226 P222 P215 | ~115 | other keyword leftovers |
| `rulings-residue-burn-freeze` | P009 P062 | ~60 | damage spells, tap-and-freeze |
| `rulings-residue-shapechange-typal` | P160 P161 P184 | ~75 | changelings, shapeshifters, typal payoffs |
| `rulings-residue-misc-1`, `-misc-2` | every other batch with OPEN texts, split in two by id | ~770 | the small residues (1–20 a batch) |

Before starting an item, re-run `list`: another item may have cited some of its rulings
(rulings shared by several cards are covered by a test on any of them). Take only batches no
other running item has taken; say which in the report.

Also folded into `rulings-residue-misc-2`, if the user wants the discrepancy log complete before
reporting (finish-line-plan decision D2): finish the release-notes comparison that
`ruling-conflicts-3` began (the ormos mirror, https://github.com/maxmakesmagic/ormos, 2013–2022;
if the host is unreachable from the build machine, say so and leave it): it hand-read about 25 of
the ~1,100 passages whose words differ from the data; read the rest for who/where/what word
changes, and log any new copying error in docs/RULING_DISCREPANCIES.md.

DONE per item when every ruling of its batches is CITED, EXEMPT with a reason or UNCOVERED with a
named reason in docs/rulings-uncovered.tsv. Report counts: cited, exempt, uncovered, bugs fixed
(each with its rule), interactions against restatements; and the row in docs/agents/yield.tsv.
