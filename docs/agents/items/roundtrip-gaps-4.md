# roundtrip-gaps-4: new gap clusters and the 4+-card wording clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 21, origin a29dfa98, after
verified-cards landed): 26,438 of 30,041 fully supported cards pass; 3,572 mismatch (new
cards keep becoming fully supported, bringing new gaps). Runs alone (one item at a time).
Round-trip failures keep cards out of the `verified` tier (docs/verified-cards.tsv), so each
fix here also raises the verified count.

CLUSTERS TAKEN (from `roundtrip --clusters 400` on a29dfa98): every `gap:` cluster, and
every wording cluster of 4 or more cards:
   24  gap: Effect  [Search for Survivors; Psychotic Episode; Underdark Rift]
   23  gap: Filter  [Pompous Gadabout; Weftwalking; Illusionary Terrain]
   18  gap: StaticEffect  [Enthralling Hold; Enduring Renewal; Diligent Farmhand]
    9  gap: Value  [Hot Pursuit; Wheel of Misfortune; Goblin Game]
    8  gap: a comparison of values of the object  [Yenna, Redtooth Regent; Sanar, Innovative First-Year; Abundant Harvest]
    3  gap: Condition  [Rocket Launcher; Shirei, Shizo's Caretaker; Expert-Level Safe]
    3  gap: PlayerModification  [Shaman's Trance; Lumbering Laundry; Detection Tower]
    3  gap: an alternative cost for spells cast with a permission  [Blue Mage's Cane; Jadzi, Oracle of Arcavios // Journey to the Oracle; The Infamous Cruelclaw]
    3  gap: an alternative cost for spells cast with a static permission  [Glimpse the Cosmos; Worldheart Phoenix; Valgavoth, Terror Eater]
    3  gap: remembering TriggerPlayer  [Sabertooth Cobra; Glass Asp; Nafs Asp]
    2  gap: a vote for objects  [Círdan the Shipwright; Mob Verdict]
    2  gap: one-use replacement effect  [Urabrask, Heretic Praetor; Moment of Silence]
    2  gap: qualities of an alternative list  [Leonardo, Sewer Samurai; Relic Axe]
    2  gap: replacement Discriminant  [Enduring Angel // Angelic Enforcer; Bitter Feud]
    1  gap: Modification  [Greymond, Avacyn's Stalwart]
    1  gap: TriggerCond  [Dream Devourer]
    1  gap: a remembered group no instruction names  [Tishana's Tidebinder]
    1  gap: remembering Union  [Eladamri, Korvecdal]
    1  gap: several remembered numbers  [The Seventh Doctor]
    1  gap: target slot 0 out of range  [Plague Reaver]
    9  oracle `the top x card of` vs rendered `a number of card from`  [Memorial Vault; Kotis, the Fangkeeper; Storybook Ride]
    8  oracle `you attacked with creature with` vs rendered `the total power of it`  [Minion of the Mighty; Werewolf Pack Leader; Hobgoblin Captain]
    5  oracle `it` vs rendered `that player`  [Soul Tithe; Pumpkin Bombs; The Fall of Kroog]
    4  oracle `each` vs rendered `your`  [Aku Djinn; Teferi, Mage of Zhalfir; Seeker of Slaanesh]
    4  oracle `from among it without paying` vs rendered `in a library from among`  [Velomachus Lorehold; Aetherworks Marvel; Kiora, Sovereign of the Deep]
    4  oracle `it is the second time` vs rendered `this is the second time`  [Belladonna Took; Saint Traft and Rem Karolus; Vito, Fanatic of Aclazotz]
    4  oracle `you discarded a card` vs rendered `there is a card you`  [Mysterio's Mirage; Gilt-Blade Prowler; Ragged Recluse // Odious Witch]
    4  oracle `{1} less to cast for` vs rendered `{x} less to cast where`  [Licia, Sanguine Tribune; Lumbering Megasloth; Valiant Changeling]

For each card: teach the renderer (`crates/mtg-engine/src/oracle/render/`) to word the AST
node from its meaning, independently of the parser. Triage every mismatch: parser bug first
(does the AST mean what the text says, in multiplayer, with control changes, tokens, copies,
replacement effects, zero-quantity choices, another player casting the card?) with a fix +
in-game test in `tests/cards/roundtrip_gaps_4.rs` that fails on the old code + a row in the
bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording, or a
CR-justified equivalence. A gap where the AST can't express the text's meaning is usually a
parser bug. Add "Parser bugs found" and "Remaining" sections (each still-mismatching card
with its reason). Regenerate docs/ROUNDTRIP.md and docs/roundtrip-passing.txt;
`roundtrip --check` must pass.

## Result

26,438 of 30,041 fully supported cards passed at the start (a29dfa98); 26,574 of 30,041
pass on the branch before the last merge, and 26,597 of 30,252 after merging the later
landings (origin's passing list at that merge has 26,361 cards); 26,599 after review
(gained-ability terms worded). 120 of the 153 cards in the taken clusters pass.

Renderer (new files `cost_amounts.rs`, `cast_from_among.rs`, `same_name_search.rs`,
`named_effects.rs`, `named_qualities.rs`, `named_statics.rs`, `named_replacements.rs`,
`behold_terms.rs`, `chosen_numbers.rs`, `object_counts.rs`, `same_is_true.rs`; small hooks
elsewhere): named behaviors put into words from what they do — same-name searches ("search
that player's graveyard, hand, and library for all cards with the same name as the chosen
card and exile them", "for each creature you control, you may search your library for a
creature card with the same name as that creature"), bidding life, reordering a graveyard,
"just beneath the top X cards of that library", revealing cards at random / a library / a
hand and the top card, "repeat this process until no one pays life", behold in its long
forms (CR 701.4a) and "if you revealed a Dragon card or chose a Dragon as you cast ~",
secret numbers ("each player with the highest number loses that much life", "If they
match"), votes for players ("each player who received no votes", "for each vote an
opponent received"), counts about an object ("with a legendary Equipment attached to it",
"that doesn't have the same name as another permanent you control", "that doesn't share a
land type with a land you control"), "for each card type among noncreature spells you've
cast this turn, you may put a card of that type ...", "choose land or nonland ... a card
of the chosen kind", "Balancing Act"'s "the player who controls the fewest ... the same
way", the static rules (spend rules, "counts as a card named", "You can't choose an
untapped creature as ~'s target as you cast it", hands revealed, face-down creatures,
hexproof ignored, echo for {0}, opponents' untap and draw steps, "can't cause you to
discard cards"), spell markers ("exile that card instead of putting it into your graveyard
as it resolves"), alternative costs of play permissions ("by paying {1} rather than paying
its mana cost") and what casting this way does ("If you do, it enters with two +1/+1
counters on it"), "skips their next combat phase this turn", "double your life total" (CR
701.10d), the life-total-reduced-to-0 and chosen-players damage replacements, and lines
cards print as one ("costs {2} less to cast and can be cast as though it had flash", "can't
attack or block, and their activated abilities with {T} in their costs can't be
activated", "Equipped creature gets +1/+1. If it's a Warrior, it gets +2/+1 instead.", "The
same is true for first strike, trample, and protection from any color."). Wording
clusters: "the top X cards of your library, where X is ..." (CR 107.3), "if you attacked
with creatures with total power 6 or greater this combat", "if you've discarded a card
this turn", "If it's the second time", "{1} less to cast for each 1 life you gained this
turn", "This effect can't reduce the amount of mana ~ costs by more than {5}", "Each
opponent must attack ...", "you may cast a spell from among them ... Put the rest on the
bottom of your library in a random order", "unless they pay", "their total power".

A delayed trigger without targets of its own now renders the creating ability's targets
(Plague Reaver's "under that player's control"), as reflexive triggers already did.

## Parser bugs found

Tests in `crates/mtg-engine/tests/cards/roundtrip_gaps_4.rs`; the bug fails on the old
code (checked by restoring the old engine files and running the tests):

- Shirei, Shizo's Caretaker: "you may return that card to the battlefield at the beginning
  of the next end step if ~ is still on the battlefield" — a one-shot delayed trigger whose
  intervening "if" was false at that end step stayed waiting and triggered at a later end
  step (CR 603.7b: only the next time its event occurs; CR 603.4), and a phased-out Shirei
  counted as still on the battlefield (CR 702.26b). `triggers.rs` (a one-shot delayed
  trigger with an intervening "if", its event matched, is spent) and
  `kw/delayed_conditions.rs`. Test: Shirei phased out at that end step and phased in later.
- Lobotomy, Shimian Specter: with no card chosen (an empty hand, only lands) nothing was
  searched and that player didn't shuffle (their rulings). Fixed here, then the same fix
  landed upstream (cr-examples-4, `kw/search_same_name.rs`, CR 701.23c) while this item
  was in progress; the merge keeps upstream's version and its tests.

Checked in play (tests pass on the old code too): Plague Reaver returns under the chosen
opponent's control at their upkeep (the delayed ability uses the creating ability's
target); Balancing Act sacrifices all players' permanents at once (Blood Artist sees the
other player's creature die with it, CR 603.10a); Shimian Specter exiles every card with
the chosen card's name; Shirei returns the card when it's still there.

## Remaining

Of the 153 cards in the taken clusters, these still mismatch; each was checked against the
AST and plays as the text says (wording only) unless noted:

- Hot Pursuit: "that creature is also goaded as long as ~ remains on the battlefield" and
  "all goaded and/or suspected creatures ... Untap them" wordings.
- Search for Survivors: "An opponent chooses a card at random" — the AST chooses the card at
  random with no chooser (a random choice is the same whoever makes it).
- Werewolf Pack Leader: the order of "has base power and toughness 5/3, gains trample, and
  isn't a Human".
- Yenna, Redtooth Regent: "Choose target enchantment ... Create a token that's a copy of it"
  rendered with the target in the instruction.
- Dream Devourer: "Each nonland card in your hand without foretell" word order.
- Wheel of Misfortune: "then all players reveal those numbers simultaneously and determine
  the highest and lowest numbers revealed this way" (determining has no game action).
- Sabertooth Cobra: "The player gets another poison counter" (the delayed instruction's
  "the player" / "another").
- Aku Djinn: "each creature each opponent controls" rendered "each creature your opponents
  control" (the same creatures).
- Glimpse the Cosmos, Valgavoth, Terror Eater: the static ability's own condition ("As long
  as you control a Giant", "During your turn") is placed after the alternative-cost
  sentence.
- Mindblaze: "If that library contains exactly the chosen number of cards with the chosen
  name" wording.
- Haunting Echoes, Reap Intellect: the searched library is the iterated card's owner's
  ("the library of that card's controller" for "that player's library"), and "You choose up
  to X nonland cards from it and exile them".
- The Seventh Doctor: the guess and two remembered numbers (`several remembered numbers`).
- Enduring Renewal: the draw replacement's "reveal the top card ... instead. If it's a
  creature card, put it into your graveyard. Otherwise, draw a card." structure.
- Goblin Game: hidden items are compiled as secretly chosen numbers of at least one (its
  rulings); the item wording isn't rendered.
- Professor Hojo: "during your turn" for the first ability each turn on your turn.
- Storybook Ride: "put them on the bottom of your library in any order" (the exiled group
  rendered as "it").
- Shaman's Trance: "can't play lands or cast spells from their graveyards" wording.
- Teferi, Who Slows the Sunset: "Untap the chosen permanents you control. Tap the chosen
  permanents you don't control." wording.
- Selective Obliteration: "unless it's colorless or it's only the color its controller
  chose".
- Sanctuary Blade: "protection from the last chosen color".
- Pumpkin Bombs, The Fall of Kroog: "they" for a target player named before (a global
  "they" for target players broke the pronoun choice of other cards).
- Celestial Dawn: "The same is true for spells you control and nonland cards you own that
  aren't on the battlefield."
- Laughing Jasper Flint: "Until end of turn, you may cast spells from among those cards, and
  mana of any type can be spent to cast those spells" (the duration's place).
- The Enigma Jewel // Locus of Enlightenment: the craft materials ("nonlands with activated
  abilities"). (The gained abilities' "only once each turn", `GainedTerms::max_per_turn`,
  was silently dropped by the renderer; review made it word both gained-ability terms,
  which also made Drana and Linvala and Grell Philosopher pass. No passing card's AST
  disagreed with its text about them.)
- Null Chamber: "you and an opponent each choose a card name other than a basic land card
  name" and the two restrictions in one sentence.
- Anzrag's Rampage: "a creature card exiled this way".
- Namor the Sub-Mariner: "create that many" for the blue mana symbols the trigger counts.
- Abundance: "you may instead choose ... put all other cards revealed this way".
- Heroic Defiance: "unless it shares a color" (rendered "as long as it doesn't share").
- Conundrum Sphinx: each player reveals and puts their card in turn; "If it doesn't, that
  player puts it on the bottom of their library".

Review notes: Celestial Dawn's "nonland cards you own that aren't on the battlefield" is
compiled for the hand, library, graveyard, exile and command zone only
(`patterns/statics_other_zones.rs`), so a spell you own that another player controls (on
the stack) isn't white; an edge left for a later item.
