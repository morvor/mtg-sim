# paper-scope-1: count every paper card as paper, in every tool

WHY: the user decided on Oct 28: "fix the tools and get to work on the 117 texts. We should ignore true digital only cards and digital versions of cards, but make sure all paper cards are represented."

**The bug.** The tools decide whether a card is a paper card from the `games` field of its Scryfall oracle record. That record is one printing. For 1,062 cards the chosen printing is an online-only reprint, so `games` lacks "paper":
- Masters Edition I–IV, Tempest Remastered and Vintage Masters account for 1,028 of them;
- MTGO duel decks, Arena starter cards and promos account for the other 34.

They are real paper cards, such as City of Traitors, Fastbond, Memnite and Raging Goblin. Yet the tools leave them out of `card-coverage`, `unsupported --paper`, `verified-cards.tsv`, the round trip, structure coverage, and the sim, fuzz and property card pools. `scripts/rulings_batches.py` counts their rulings as DEFERRED (digital-only).

**The rule.** A card is a paper card if either of these holds:
- its oracle record's `games` is empty or includes "paper";
- it is legal, restricted or banned in at least one format played with paper cards: standard, pioneer, modern, legacy, vintage, pauper, commander, oathbreaker, paupercommander, duel, oldschool, premodern, predh.

This leaves out exactly what the user wants left out:
- true digital-only cards: Alchemy and Arena-only sets, legal only in Arena formats (alchemy, historic, timeless, brawl, standardbrawl, gladiator, competitivebrawl). On Oct 28 that was 657 cards, from sets such as hbg, the y-sets and j21;
- digital versions of cards (the "A-" rebalanced Alchemy cards).

Checked on the Oct 28 data: all 1,062 cards the rule adds are paper reprints, by their sets (me1–4, tpr, vma, td2, oana, aa2, prm, pz2, unf).

Exact printing data (Scryfall's `default_cards` bulk) would also settle it, but the environment's network proxy blocks Scryfall (CONNECT 403). Use the rule. Add a test that pins it on known cards:
- paper: City of Traitors, Fastbond, Memnite, Raging Goblin, Pyrogoyf;
- not paper: an Alchemy "A-" card, a j21 card.

**Tasks:**
1. Put the rule in one place: `ScryfallCard::is_paper()` in crates/mtg-data/src/scryfall.rs, with the format list and a comment saying why. Use it everywhere the copied `games` check now stands. Today that is:
   - crates/mtg-tools: main.rs card-coverage and unsupported, roundtrip.rs, structure_coverage.rs;
   - crates/mtg-sim: decks.rs, every_card.rs;
   - crates/mtg-engine/src/properties: mod.rs, copies.rs;
   - crates/mtg-api/tests/common;
   - any other place `rg '"paper"'` finds;
   - `deferred()` in scripts/rulings_batches.py, the same rule in Python.
2. Regenerate the documents whose counts change, and state each before/after count in the commit and the report:
   - docs/CARD_COVERAGE.md;
   - docs/ROUNDTRIP.md and its passing list;
   - docs/STRUCTURE_COVERAGE.md;
   - the `rulings_batches.py summary` line;
   - DEFERRED.md's "Digital-only cards" section: its count, and that the section means cards with no paper printing and digital versions of cards.

   `docs/verified-cards.tsv` gains the newly counted cards with the "fuzz not swept" check failing. Sweep just the newly counted, fully supported cards if the every-card mode can take a subset. Otherwise leave them for the next full regeneration, and say which.
3. Refresh `docs/agents/round4/paper-scope-gap.tsv` from the fixed `mtg-tools unsupported --paper --by-card`. List every newly counted unsupported text that no round-3 or round-4 list places, in the same columns. Item `cards4-paper-plan` plans those texts, and it can start from the current file.

DONE when every tool and script uses the one rule, the coverage documents show the corrected counts, DEFERRED.md is accurate, paper-scope-gap.tsv is current, and `cargo test --workspace` passes.
