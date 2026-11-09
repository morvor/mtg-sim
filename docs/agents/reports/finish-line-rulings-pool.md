# The open rulings pool, sized for the finish-line plan (Nov 9, 2026)

Read by a subagent of the `finish-line-plan` planner with scripts/rulings_batches.py; the plan's phase A (docs/agents/finish-line-plan.md §3) orders the batches from this.


Source: `scripts/rulings_batches.py` (plan() + status_fn imported directly), docs/verified-cards.tsv
(tier column), crates/mtg-engine/tests/rulings/r_pNNN_*.rs (which batches were ever worked), git log
(shallow clone: history starts 2026-11-05 12:53 UTC, 196 commits), docs/agents/verification-plan.md.
Summary today: 33,955 texts / 227 batches: CITED 25,777 (75.9%), EXEMPT 189, OPEN 6,125, DEFERRED 1,864.

## 1. Where the 6,125 OPEN rulings are

- 154 of 227 batches have OPEN rulings; 73 are at zero.
- **40 batches have never been worked** (no `tests/rulings/r_pNNN_*.rs`): P049 P078 P081 P083 P084 P086 P088
  P090 P093 P095 P099 P100 P101 P103 P105 P106 P114 P115 P135 P138 P139 P144 P149 P153 P154 P155 P157 P158
  P164 P167 P168 P170 P173 P175 P181 P182 P191 P196 P197 P200. They hold **4,526 OPEN** (86–131 each, mean 113;
  each already has 17–53 CITED from other batches' tests).
- **114 worked batches keep a residue of 1,599 OPEN**: 29 batches with 20–43 (845 total), 40 with 10–19 (582),
  45 with 1–9 (172). The 23 batches landed since Nov 5 left 1 OPEN in total, so the residue is from older
  rounds (keyword sweeps, pre-"drive to zero" rounds), chiefly copy-creature P020/P021/P023 (116), reflexive/
  refund P130 (42), counters P035/P120 (71), keyword batches P217/P211/P209/P208/P207/P210/P212/P213 (~165),
  type change P187/P188 (58), hate-attacker/blocker P079/P080/P082 (63), copy-instant P028/P030/P032 (47).
- **Unsupported cards are not the blocker**: only 1 OPEN ruling applies solely to an unsupported-tier card
  (P076, Tenacious Pup, "a one-time boon is a single-use triggered ability"). The tier file has 35 unsupported
  cards in all. Best tier among each OPEN ruling's cards: verified 2,097, round-trip 2,135, compiled-only 1,893
  (compiled-only = compiles but fails round-trip or the every-card sweep; still usable in TestGame), unsupported 1.
  6,052 OPEN texts belong to exactly one card; 73 are shared.

Top 40 by OPEN (= exactly the 40 never-worked batches). Columns: id, OPEN, CITED, OPEN-only-unsupported, groups.

| id | OPEN | CITED | unsup | grouping (OPEN counts) |
|---|---|---|---|---|
| P144 | 131 | 19 | 0 | repeatable impulsive draw 130 |
| P081 | 126 | 21 | 0 | hate-blocker 47; hate-attacker 43; hate-black 36 |
| P154 | 126 | 18 | 0 | scales with mana value 126 |
| P139 | 125 | 21 | 0 | removal-sacrifice 94; removal-toughness 23; removal-tuck 8 |
| P157 | 125 | 22 | 0 | selective group hug 56; self-replacement effect 51; self life loss matters 11 |
| P086 | 124 | 26 | 0 | hate-set-mechanic 80; hate-regenerate 20; hate-noncreature 12; hate-red 10 |
| P155 | 124 | 24 | 0 | scales with mana value 105; scales with power 13 |
| P101 | 123 | 22 | 0 | manaless value 72; mana value matters 48 |
| P182 | 123 | 17 | 0 | tutor-land-to-battlefield 51; tutor-to-battlefield 44; tutor-land-basic/plains 11 |
| P138 | 121 | 26 | 0 | removal-sacrifice 121 |
| P170 | 121 | 24 | 0 | synergy-noncreature 69; synergy-planeswalker 47; synergy-party 5 |
| P084 | 120 | 29 | 0 | hate-graveyard 120 |
| P153 | 120 | 30 | 0 | scales with mana value 120 |
| P168 | 120 | 26 | 0 | synergy-legendary 59; synergy-green 38; synergy-island 13 |
| P191 | 118 | 28 | 0 | unique mana cost 118 |
| P106 | 117 | 32 | 0 | mini refund 117 |
| P135 | 117 | 25 | 0 | removal-land 78; removal-fight 39 |
| P158 | 116 | 33 | 0 | self-replacement effect 116 |
| P090 | 115 | 24 | 0 | jump 18; impulse-creature 17; inverted effects 13; infusion 12 (+ many small) |
| P095 | 115 | 31 | 0 | life payment 65; lifegain matters 19; life-total-matters-self 11; lifegain 11 |
| P100 | 115 | 28 | 0 | mana value matters 115 |
| P167 | 114 | 31 | 0 | synergy-equipment 46; synergy-forest 21; synergy-green 21; synergy-flying 9 |
| P173 | 113 | 30 | 0 | tap fuel-creature 108 |
| P083 | 112 | 33 | 0 | hate-discard 33; hate-counterspell 25; hate-flying 18; hate-graveyard 16 |
| P093 | 112 | 33 | 0 | life for cards 110 |
| P200 | 112 | 36 | 0 | virtual legendary 61; weaker in singleton formats 18; warlord 9; werewolf 8 |
| P049 | 111 | 32 | 0 | drain life 91; draft matters 13; drain creature 6 |
| P114 | 111 | 34 | 0 | pair-commander 111 |
| P196 | 111 | 22 | 0 | untracked indefinite effect 111 |
| P197 | 110 | 23 | 0 | untracked indefinite effect 110 |
| P103 | 108 | 26 | 0 | mill-any 63; mill-opponent 25; meme 8; mill-exile 8 |
| P088 | 107 | 40 | 0 | hatebear 50; heroic 30; hellbending 13; high mana value matters 6 |
| P181 | 105 | 29 | 0 | tutor-land-basic 82; tutor-card 9; tutor-creature 9 |
| P175 | 101 | 30 | 0 | temporary token 52; tapper-permanent 14; temporary reanimation 12; theft-cast 8 |
| P078 | 98 | 35 | 0 | group hug 34; hand size matters 24; group slug 10; graveyard seal 9 |
| P164 | 96 | 46 | 0 | synergy-black 30; synergy-aura 23; synergy-blocker-self 17; synergy-activated-ability 13 |
| P099 | 95 | 45 | 0 | mana value matters 28; mana filter 25; mana increaser 21 |
| P105 | 93 | 53 | 0 | mill-self 71; mini refund 21 |
| P149 | 89 | 33 | 0 | rhystic 81; ritual 4; roll d6 3 |
| P115 | 86 | 48 | 0 | peek-hand 25; pair-commander 12; per-player 10; pinger 10; phasing |

Full list of all 154 batches with OPEN (tab-separated: batch, open, cited, exempt, deferred,
open_all_unsupported, top_open_groups): same directory, `batches_open.tsv`; every OPEN ruling with its cards and
tiers: `open_rulings.json`; the 60-ruling sample: `sample60.txt`.

## 2. Assignment and what has landed

- Landed with `rulings-PNNN` subjects in the visible history (Nov 5–8): P008 P019 P022 P029 P033 P040 P041 P042
  P043 P047 P048 P050 P055 P059 P060 P061 P067 P068 P069 P070 P071 P077 P087 (23). Everything before Nov 5 12:53
  is outside the shallow clone; the test-file census (187 worked batches) is the reliable "landed" signal.
- The Nov 7 API launcher queue (P061 P067 P068 P069 P070 P071 P077 P078 P081 ...) landed through P077; P078 onward
  is untouched. The Nov 9 re-plan (verification-plan.md) lists rulings-1 = odd positions, rulings-2 = even positions of the pool in
  id order (its examples P049, P078, P081 ... match the never-worked list exactly). verification-plan.md line 50's
  older ordered pool is superseded (every batch it names before P049 has landed).
- **OPEN batches not yet landed, in pool order (id order of never-worked batches):**
  rulings-1 (odd): P049 P081 P084 P088 P093 P099 P101 P105 P114 P135 P139 P149 P154 P157 P164 P168 P173 P181 P191 P197
  rulings-2 (even): P078 P083 P086 P090 P095 P100 P103 P106 P115 P138 P144 P153 P155 P158 P167 P170 P175 P182 P196 P200
  Plus the residue in 114 worked batches (section 1), which no queue currently covers.

## 3. Rate and yield

Batches landed per day (git, shallow): Nov 5: 16 (12:53–18:30 UTC, ~6 parallel subscription workers), Nov 6: 3,
Nov 7: 0 (workers held), Nov 8: 7 (API launcher, 2 children, 03:24–11:00: 1.1–1.7 h per batch per child, ~1
batch/h aggregate), Nov 9: 0 (re-planning). Plan notes for the earlier period: Nov 1 API run, 16 batches in ~10 h
for $183 (~$11.50/batch). A batch = one implementer + one review pass, 1–2 h wall clock per worker.

Final-state split of the 23 recent batches: 3,258 CITED, 12 EXEMPT (0.4%), 1 OPEN, 179 DEFERRED — recent batches
drive OPEN to zero; exemptions are rare (189 of 33,955 overall, 0.6%). Commit bodies carry no count lines
(reports went to the coordinator), so the split comes from the data.

Engine fixes in those 23 batches (commits touching crates/mtg-engine/src, excluding merges/fmt):
core engine (~20): P008 cost-discarded cards in a hidden zone counted as they last existed; P022 unflipped copy of a
flipped flip card uses its normal side (layers/flip), Hofri's Spirits keep the card their resolution exiled and
links to a creator that left; P033 X is 0 when an alternative activation cost without {X} is paid; P042 abilities
of an object that left its zone can't be activated; P043 morph/disguise cast options clone characteristics only
for face-down cards, shields divided by recipient; P047 target checks ignore what an Equipment was attached to;
P050 reflexive trigger after putting N counters triggers once per N put, count computed only when counters were
to be put; P055 drafting with several Cogwork Librarians at once; P060 a draw replaced by draws counts as drawn
this way, a player named twice in "you and that player each draw" draws twice, a draw another player made
instead isn't one drawn this way; P061 protection referring to an undefined choice grants nothing (Haktos);
P067 Opposition Agent exiles every card a search found in any zone and the latest Agent's controller chooses,
a single searcher inside "for each player" searches the iterated player's library; P068 a choice of a spell to
cast from several zones judges each card as the spell; P069 faces chosen kept when no total-MV limit applies,
spells under a total-MV limit fit it as the spells they're cast as; P070 an X the text defines is the only value
offered; P071 a permission's keyword rider (undaunted) counts when listing castable cards; P077 escape exiles
cards its other costs put into the graveyard, an exiled card's power read from the graveyard.
Compiler/pattern fixes (~12): P050 Kaya gains the amount it says not the damage dealt; P059 Phelia's "if it
entered under your control" is part of the delayed return; P060 each-player follow-ups; P061 trailing "for each
card drawn this way" after ", then" scales only the last instruction; P069 Reanimate's life loss uses MV in the
graveyard; P070 Kefka's random card from each opponent's graveyard is one from each; P087 per-player damage
amounts and class restrictions for a target player's creatures.
Zero-fix batches: P019 P029 P040 P041 P048 (5 of 23).
=> ~0.9 core engine bugs + ~0.5 compiler fixes per batch of ~100–140 processed rulings: worker.md's "~1 real
engine bug per 100 rulings" holds (0.7–1.0 per 100). The cast-from-exile family P067–P071 alone yielded ~10.

## 4. Character of the OPEN pool (random sample of 60, seed 20261109; see sample60.txt)

- (a) restates the card's text: 15 = **25%** (e.g. Terrarion triggers however it's put into a graveyard; Syr
  Gwyn triggers once per equipped attacker; Library of Leng: milled cards aren't discarded; Koma can target a
  tapped permanent; Harness Infinity exchanges with an empty zone).
- (b) interaction: 45 = **75%** — LKI/source-leaves-before-resolution 11 (Nihiloor, Cartographer's Hawk, Dusk
  Rose Reliquary, Shadow, Scaled Nurturer, Sabertooth Cobra...), costs/alternative costs/MV 9 (Sundering Stroke,
  Oracle's Vault, Cantankerous Keepers, Ravenous Trap, Demon of Death's Gate, Eldritch Evolution, Sudden
  Insight...), timing/priority/mid-resolution 7 (Benefactor's Draught, Sylvan Library, Naya Soulbeast, Hope of
  Ghirapur, Barl's Cage...), copies 4 (Orvar, Mycosynth Gardens, Aethermage's Touch, Altered-Ego-style), zones/
  new objects 4 (Endless Horizons, Stalking Leonin, Abdel Adrian...), replacement/durations 4 (Island Sanctuary,
  Undead Alchemist, Soulstone Sanctuary, Orzhov Advokist), fizzle/illegal target 3 (Urza's Rage, Ravager Wurm,
  Beamsaw Prospector), multiplayer 2 (Grave Pact 2HG, Idolized attacks alone), layers/damage/phasing 3.
  About a third of (b) are stock patterns (source leaves, new object, fizzle) that cost little once the
  batch has a helper.
- (c) about an unsupported card: 0 = **0%** (pool-wide: 1 of 6,125).
- (d) presentation/policy (would be exempt): 0 = **0%** (historical exempt rate 0.4–0.6%).
Restatement share by batch in the sample: P093 3 of 5, P086 2 of 3, P149 1 of 3, P138 1 of 3; the MV/cost,
replacement, tutor and impulsive-draw batches were all interaction.

## 5. Estimate

- Work items: 40 never-worked batches (4,526 OPEN) + the 1,599-ruling residue regrouped into ~14 themed items of
  ~100–120 (copy-creature P020+P021+P023; Enchant/Equip P207–P213; counters P035/P120/P119/P121; reflexive/refund
  P129/P130; type change P187/P188; hate-attacker/blocker P079/P080/P082; copy-instant P028/P030/P032; keyword
  leftovers P217/P211/P214/P205/P226/P222/P215; P009 burn; P062 freeze; P161/P160 shapechange; P184 typal;
  small residues swept in two "residue-misc" items). **≈54 implementer sessions** (each with its review pass).
- Throughput at the measured rate (1–2 h per batch per worker; 7 batches per 8 h from 2 API children; 16 per
  ~6 h from 6 workers): two rulings sessions ≈ 10–14 batches/day if they loop continuously → **4–5 days**;
  adding the API launcher's 2 children brings it to ~2–3 days; the Nov 5 configuration (6 workers) ≈ 2 days.
  Residue items are slower per ruling (what earlier rounds left was the harder or then-unsupported part).
- Bugs: 0.7–1.0 core engine bugs per 100 processed (measured on batches that also contained restatements), and the
  open pool is 75% interaction (vs. 25% restatement), so expect **≈45–60 core engine bugs plus ≈30 compiler/pattern
  fixes** from the 6,125; roughly 35–45 of the engine bugs are in the 40 never-worked batches.
- Do first (interaction-heavy): P154 P153 P155 P100 P101 (mana value and costs: X, DFC/meld/copy MV, alternative
  costs, LKI of sacrificed permanents), P158 P157 (self-replacement effects, "without paying its mana cost"),
  P144 P090 (impulsive draw / impulse-creature: exile permissions and costs, the P067–P071 family that found ~10
  bugs), P196 P197 (untracked indefinite effects: durations, 611.2, leaves-battlefield), P138 P139 (removal-
  sacrifice: triggers, LKI, 2HG, enters attacking), P182 P181 (tutor to battlefield: Auras attaching, simultaneous
  entry, fizzles), P175 (temporary tokens/reanimation/theft: control and zones), P114 (pair-commander), P200
  (virtual legendary/copies), P115 (phasing, peek), P135 (fight with illegal targets, land removal), and the
  copy-creature residue P020/P021/P023.
- Cheap, table-driven (mostly restatements): P093 P095 P049 (life payment/drain), P081 P083 P084 P086 (hate-X:
  target legality, "doesn't stop/affect"), P164 P167 P168 P170 P173 (synergy-X counting), P103 P105 P106 (mill,
  refund), P191 (unique mana cost), P149 (rhystic), P088 (hatebear/heroic), P078 (group hug/hand size) — though
  each still carries ~50% interactions (Grave Pact 2HG and Undead Alchemist sat in "removal-sacrifice"/"mill").
