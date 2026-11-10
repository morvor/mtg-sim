# register-sweep-1: every behavioral deviation in the register, the equivalence list read, one rulings number

WHY: docs/agents/finish-line-plan.md, criterion C8 and risks 10, 11. The interpretation register
(docs/INTERPRETATIONS.md, 128 entries, 0 provisional) is the user's check on the judgment calls
workers made. Behavioral differences can live only in an item's "Remaining" section instead:
Inquisitor Eisenhorn's (roundtrip-tail-5: a card drawn while casting is revealed only after the
cast, so its "this way" ability never triggers) turned out, on reading CR 121.8, to be a bug
rather than a reading (the card is revealed when the spell becomes cast, and the trigger should
fire then; gaps-residual-2 fixes it); others may be readings that belong in the register. The
renderer's equivalence list (`crates/mtg-engine/src/oracle/render/compare.rs`, `EQUIVALENCES`,
about 150 rules plus named approximations such as "cycle or discard" read as discard under
702.29d) is where a real difference could be normalized away, and nobody has read it as a whole
against the CR. And the committed docs/RULINGS_COVERAGE.md (1,558 unique texts, 4.6%) was never regenerated:
`mtg-tools rulings-coverage --check` agrees with `scripts/rulings_batches.py` on the cited count
(25,777) but counts 34,000 texts and 166 exempt where the script counts 33,955 in-scope texts
and 189 exempt, so the standard update has two sets of numbers.

Run the register work at effort max; the rest is routine. This item changes no engine behavior.

## Work

1. **Sweep the items for unregistered deviations.** Read every "Remaining", "Not done",
   "unless noted", "deviation", "approximation" and "we chose" passage in
   docs/agents/items/*.md, docs/agents/round*/*.md, docs/ROUNDTRIP.md, docs/PROPERTIES.md
   ("Remaining gaps"), docs/EVERY_CARD.md and docs/MUTATION.md (survivors called "equivalent for
   today's cards"). For each behavior the engine shows that a reasonable judge could read the CR
   differently on, or that knowingly deviates from the CR: add a register entry (provisional,
   with the CR text quoted and the alternatives) or cite the existing one, and pin it with a
   test citing `interp!("<id>")`. A difference the CR settles plainly is a bug, not a reading:
   hand it to `gaps-residual-2` (or fix it when small) rather than register it. Also the two
   medium-confidence judge calls from audit 1 (cr-04, rulings-18) if they are not registered
   yet.
2. **Read EQUIVALENCES against the CR.** For each rule and each named approximation: cite the
   CR rule that makes the two wordings the same (in the list's comment); or, where no rules
   content differs (a grammatical or number-word variant), mark it "wording only", which the
   reviewer confirms; or, if it hides a difference, remove it, let the affected cards mismatch,
   and list them for the next `roundtrip-pairs-N` item. Never cite a rule that doesn't make the
   wordings equivalent just to fill the slot. Report every rule removed with the cards it
   affected.
3. **One rulings number.** Regenerate docs/RULINGS_COVERAGE.md; make `mtg-tools
   rulings-coverage` and the batches script agree on the denominator (which texts are in scope:
   the tool counts 34,000, the script 33,955 after the Contraption and host/augment texts) and
   on the exempt count (166 against 189: find which exemption lines one side skips), with
   DEFERRED and out-of-scope texts reported separately; add a test that compares the two on the
   committed data so they stay in step.

## Report

The register entries added (each with its quotes; the provisional ones go to the user), the
equivalence rules cited or removed, and the reconciled rulings numbers.

DONE when the sweep finds nothing unregistered, every EQUIVALENCES rule carries a CR citation or
a confirmed "wording only" note, and the two rulings counts agree.

## Result (Nov 10)

Commits `register-sweep-1: ...` on wip/register-sweep-1. No engine behavior changed.

**1. Sweep.** Read in full: every "Remaining", "STATUS", "Review" and "NOT TAKEN" section
and every "unless noted", "deviation", "approximation", "we chose", "left as is", "not
modeled" line of docs/agents/items/*.md (88 files have one), docs/ROUNDTRIP.md,
docs/PROPERTIES.md "Remaining gaps", docs/EVERY_CARD.md and every survivor of
docs/MUTATION.md; docs/agents/round*/ holds no .md files (tsv and py only). Findings:

- No unregistered reading or deviation. The one behavioral difference living only in a
  "Remaining" section is Inquisitor Eisenhorn's (roundtrip-tail-5; CR 121.8), already a bug
  for `gaps-residual-2`, not a reading. Every other "unless noted" case is wording on reading
  its AST note, except two with a structural difference, verified in play or code: "you may
  discard/exile one or more ... When you do, ..." compiled as "any number" (Loamcrafter Faun,
  Specter of Mortality) — choosing no card is declining, the reflexive ability doesn't trigger
  (`tests/cards/register_sweep_1.rs`); and Celestial Dawn's "nonland cards you own that aren't
  on the battlefield" (roundtrip-gaps-4's edge, a spell you own on the stack) has since been
  compiled for the stack too (`patterns/statics_other_zones.rs`, `NOT_ON_BATTLEFIELD`).
- Behaviors the sweep met that are already registered and cited: Lotus Vale's one-untapped-land
  case (`enter-sacrifice-instead-partial`), a cost-determined divided amount
  (`division-after-cost-amount`), Words of War's target (`words-target-on-activation`),
  Backdraft (`backdraft-controller-chooses-the-spell`), The Toymaker's Trap
  (`toymakers-trap-every-number-chosen`), Uldaros's copies (`total-mana-value-choice-fitted`),
  outside the game (`outside-the-game-is-the-sideboard`), a set name
  (`name-setting-in-layer-3`), Jeweled Amulet (`jeweled-amulet-several-types`), growing loops
  (`growing-loops-run-to-the-limit`), ability ids (`ability-ids-depend-on-the-process`), the
  733.1 reversal (`illegal-action-reversal-restores-libraries`), simultaneous timestamps (the
  player's choice per 613.7m, defaulting by `default-order-as-listed`); Rock Jockey's "you"
  (CR 109.5), Goblin Game's items (its rulings) and Cryptic Spires' circled colors (CR 207.5)
  need no entry. Audit 1's cr-04 and rulings-18 are `conspiracy-draft-needs-three-players`
  and `copy-exception-ability-stacks`, both cited.
- docs/MUTATION.md's "equivalent with today's cards / in practice / for in-scope cards"
  survivors are mutants equivalent on the reachable paths, not engine readings (the engine
  side of each was read: permissions.rs:425, simultaneous.rs:766, mana.rs:214,
  replacement.rs:1600, triggers.rs:688, layers.rs:1489/1550/1519).
- Register: 128 entries, 0 provisional, unchanged; `interpretations --check` passes.

**2. EQUIVALENCES** (`crates/mtg-engine/src/oracle/render/compare.rs`): 164 rules read against
the CR text. 42 cite the CR rule (or ruling) that makes the two wordings the same; 122 are
marked "Wording only" (grammar, anaphora, word order, number, tense, contraction, synonym),
for the reviewer to confirm. Citations that didn't make the wordings equivalent were dropped
(506.4, 106.1, 121.1, 115.3, 122.1, 602.5b, 400.6, ...) and loose ones corrected (118.12a for
"unless", 509.1c for a cost to block, 111.3 for a token's "It has", 701.18b for "play a
card", 109.5 for a subjectless instruction, 119.3–119.5 for "life total can't change",
Malcolm's and Myth Unbound's rulings). SENTENCE_FORMS (12) and IGNORED_WORDS (2) carry the
same notes; dropping "and" is recorded as a known limit ("all artifacts and creatures" and
"all artifact creatures" compare alike).
- Removed (1): "with the same name as ~" read as "named ~": an object's current name and its
  printed name differ (CR 201.2a, 201.5: Spy Kit, clones). It exposed six cards whose grandeur
  cost "Discard another card named ~" is compiled as the current name while every other
  "named ~" compiles to the printed name (Squadron Hawk, Relentless Rats): Baru, Fist of
  Krosa; Korlash, Heir to Blackblade; Oriss, Samite Guardian; Page, Loose Leaf; Skoa,
  Embermage; Tarox Bladewing. They mismatch now (28,455 of 32,076 pass, from 28,460) and are
  for the next `roundtrip-pairs-N`: compile the cost's "card named ~" as the printed name.
- Narrowed (1): a token's following "It gains '...'" is no longer read as its text (Timothar,
  Baron of Bats's 2021-11-19 ruling: a copy of the Bat token doesn't have the granted
  ability); "It has" stays (CR 111.3). Added (1): the renderer's `it {alt:gains|has}` for an
  ability given for a duration is "it has" (CR 611.2a, 613.1f; Mathas, Obsidian Fireheart,
  Makeshift Mannequin). Rewritten (1): Pirated Copy's "or another creature with the same
  name" becomes the explicit "with the same name as ~" (the current name) instead of
  "named ~". The gendered-pronoun rules were checked against every legal card using one
  (126 cards): each refers to the card itself. The one named approximation ("cycle or
  discard" compiled as discard) stands on CR 702.29a and 702.29d.
- A limit of the comparison, not of the list: all anaphora normalize to one token, so a
  pronoun bound to the wrong one of two objects other than ~ passes (roundtrip-probe-1).

**3. One rulings number.** `mtg-tools rulings-coverage` counted every unique text in the
data (34,000) and exempted the named card's texts (166); `scripts/rulings_batches.py`
counted in-scope texts (33,955) and exempted every text containing any needle (189). The
45 texts of difference are found only on excluded cards (21 Contraptions, 19 host/augment, 3
emblems, 2 tokens); the 23 exempt texts of difference matched a needle written for another
card (5 Duel Commander bans, 5 "in a casual game, a card from outside the game", face-down
and same-name differentiation, a hat, an artist credit, story spotlights, tournament rules
for a controlled player, the miracle reveal, and 5 testable rulings: Etrata, the Silencer;
Ox Drover; Kogla and Yidaro; Yawgmoth, Thran Physician; Phantasmal Form, through Acolyte
Hybrid's "and you won't draw a card" and Blitz Hellion's "its owner doesn't shuffle their
library"). Both tools now count the texts of playable cards not out of scope, mark DEFERRED
the texts every card of which is deferred, read an exemption line as the named card's
rulings (shared texts exempt everywhere), and print the same
`rulings: texts=33955 cited=25777 exempt=184 open=6130 deferred=1864 excluded=45`
(`crates/mtg-tools/tests/rulings_counts.rs` compares the two lines; the 18 presentation and
policy texts got explicit lines in their batches' files, the 5 testable ones are OPEN).
docs/RULINGS_COVERAGE.md regenerated (it had 1,558 cited texts, from October).
