# roundtrip-tail-6: every cluster of 3 or more cards, plus Celestial Dawn

Part of the `roundtrip-followups.md` stream. Baseline (Oct 21, origin 438a3bac, after
roundtrip-gaps-4 landed): 26,599 of 30,252 fully supported cards pass; 3,523 mismatch, in
299 single-card and 264 two-card clusters and the clusters below. Runs alone.

CLUSTERS TAKEN (every cluster of 3+ cards from `roundtrip --clusters 600` on 438a3bac):
    7  gap: Filter  [Undercover Operative; Flesh Duplicate; Radiate]
    6  gap: Effect  [Adeline, Resplendent Cathar; Gollum, Scheming Guide; The Seventh Doctor]
    4  oracle `choose a nonland card from` vs rendered `{opt:you} choose a nonland card`  [Agonizing Remorse; Memory Theft; Check for Traps]
    4  oracle `choose a nonland card in` vs rendered `{opt:you} choose {alt:a nonland card that player owns or graveyard|a nonland card that player owns in a hand or a nonland card that player owns in a graveyard} from it`  [Memory Leak; Covetous Urge; Never Happened]
    3  gap: replacement Discriminant  [Mirrormind Crown; Esix, Fractal Bloom; Moonlit Meditation]
    3  oracle `a` vs rendered `other`  [Living Lore; Dermotaxi; Stormwing Entity]
    3  oracle `dy or is put to` vs rendered `{alt:dies|is put to a graveyard} or is put to`  [Reyhan, Last of the Abzan; Syr Vondam, Sunstar Exemplar; Kaya's Ghostform]
    3  oracle `from among card you own` vs rendered `exiled with ~it you own`  [Dawnhand Dissident; Boiling Rock Rioter; Intrepid Paleontologist]
    3  oracle `from among card` vs rendered `∅`  [Kaho, Minamo Historian; Jeleva, Nephalia's Scourge; Shell of the Last Kappa]
    3  oracle `if you would draw a` vs rendered `optionally if you would draw`  [Pursuit of Knowledge; Abundance; Obstinate Familiar]
    3  oracle `it` vs rendered `defending player`  [Guiltfeeder; Unquenchable Fury; Generous Plunderer]
    3  oracle `last` vs rendered `∅`  [Triarch Stalker; Sanctuary Blade; Beckoning Will-o'-Wisp]
    3  oracle `may put any number of` vs rendered `{opt:may} put any number of`  [Kamahl's Druidic Vow; Nashi, Searcher in the Dark; Blex, Vexing Pest // Search for Blex]
    3  oracle `no mana was spent to` vs rendered `the amount of mana spent`  [Roiling Vortex; Vexing Bauble; Boromir, Warden of the Tower]
    3  oracle `of your` vs rendered `∅`  [Aurelia, the Law Above; Molten Lavamancer; Everything Pizza]
    3  oracle `on` vs rendered `to cast`  [Domri, Chaos Bringer; Hall of the Bandit Lord; Carnelian Orb of Dragonkind]
    3  oracle `put the rest on the` vs rendered `{opt:under your control} put the rest on`  [Audacious Reshapers; Vivien's Invocation; Madcap Experiment]
    3  oracle `target opponent exile a creature` vs rendered `exile a creature target opponent`  [Strategic Betrayal; Doomfall; Debt to the Kami]
    3  oracle `that is` vs rendered `∅`  [Crowd of True Believers; Arc Spitter; Labyrinth Raptor]
    3  oracle `that opponent` vs rendered `{alt:that player|it}`  [Zhang Liao, Hero of Hefei; Parapet Thrasher; Fungal Shambler]
    3  oracle `that player may choose any` vs rendered `{alt:that player|it} choose any number of`  [Magnetic Mountain; Dream Tides; Thelon's Curse]
    3  oracle `the chosen card to owner` vs rendered `it to owner graveyard put`  [Gifts Ungiven; Elemental Teachings; Realms Uncharted]
    3  oracle `this turn without paying it` vs rendered `without paying it mana cost`  [Quintorius, Loremaster; Caves of Chaos Adventurer; Golos, Tireless Pilgrim]
    3  oracle `token that is a copy` vs rendered `tapped attacking token that is`  [Thousand-Faced Shadow; Shredder, Shadow Master; Flamerush Rider]
    3  oracle `you have less than or` vs rendered `2 time you have less`  [Bhaal, Lord of Murder; Myrkul, Lord of Bones; Bane, Lord of Darkness]
    3  oracle `you` vs rendered `{alt:while|if} {alt:that player|it} is a you`  [Sabotage Strategist; Reveille Squad; Orim's Prayer]
    3  oracle `your team control other warrior` vs rendered `there is other warrior you`  [Sickle Dancer; Aurora Champion; Bull-Rush Bruiser]
    3  oracle `~` vs rendered `it`  [Skeletonize; Gluttonous Hellkite; Henge Walker]
    3  oracle `∅` vs rendered `card`  [Definitely Not a Turtle; Phyrexian Censor; Deafening Silence]
    3  oracle `∅` vs rendered `have ~`  [Hardened Berserker; Invasion of the Giants; Peerless Samurai]
    3  oracle `∅` vs rendered `in addition to it other`  [Stonework Packbeast; Tajuru Paragon; Veteran Adventurer]
    3  oracle `∅` vs rendered `it`  [Atomic Microsizer; Jade Orb of Dragonkind; Sway of the Stars]
    3  oracle `∅` vs rendered `mana value {opt:equal to}`  [Skyfire Kirin; Infernal Kirin; Celestial Kirin]
    3  oracle `∅` vs rendered `on it with`  [Dust Animus; Voidpouncer; Perennation]
    3  oracle `∅` vs rendered `sacrifice`  [Glass-Cast Heart; Kyscu Drake; Magus of the Order]
    3  oracle `∅` vs rendered `this turn`  [Torch the Tower; Animal Attendant; Brutal Hordechief]
    3  oracle `∅` vs rendered `{alt:for as long as it remains exiled|for as long as it remain exiled}`  [Furious Rise; Superior Foes of Spider-Man; Unstable Amulet]

Some of these cards were listed as "Remaining (wording only)" by earlier items (Kaho, Jeleva,
Nashi, Blex, Kamahl's Druidic Vow, Sanctuary Blade, ...): earlier triage says their AST is
right, so the job there is renderer wording — but re-check anything the earlier note didn't
cover.

ALSO TAKEN: Celestial Dawn's "nonland cards you own that aren't on the battlefield" leaves
out the stack (`patterns/statics_other_zones.rs`), so a spell you own that another player
controls isn't made white (found by the roundtrip-gaps-4 reviewer). Fix with an in-game test.

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects, zero-quantity
choices?) with a fix + in-game test in `tests/cards/roundtrip_tail_6.rs` that fails on the
old code + a row in the bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else
renderer wording (independently of the parser), or a CR-justified equivalence. Add "Parser
bugs found" and "Remaining" sections. Regenerate docs/ROUNDTRIP.md and
docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Result

26,599 of 30,252 fully supported cards passed at the start (438a3bac); 26,738 of 30,252
pass on the branch before the last merge, and 26,756 of 30,412 after merging the later
landings (origin's passing list at that merge has 26,599 cards). 112 of the 121 cards taken
(the 120 in the clusters, and Celestial Dawn) pass.

Renderer (new files `guesses.rs`, `token_copies.rs`, `off_battlefield.rs`; small hooks
elsewhere): guesses ("[player] guesses whether [proposition]", "if they guessed wrong", "If
you don't cast a spell this way"), "the first time you would create one or more tokens each
turn, you may instead create that many tokens that are copies of ...", the static abilities
about "cards you own that aren't on the battlefield" ("The same is true for spells you
control and ..."), copy exceptions on the copied object ("if that creature doesn't have
vanishing"), "copy that spell for each other permanent or player the spell could target",
"if you control one or more other creatures that spell could target, choose one of those
creatures. Copy that spell. The copy targets the chosen creature", tokens "tapped and
attacking that player or a planeswalker they control", "you choose a nonland card from that
player's graveyard or hand" / "from it or a card from their graveyard", "that player may
choose any number of tapped blue creatures they control and pay {4} for each creature chosen
this way", optional draw replacements ("you may skip that draw instead", "you may instead
..."), "whenever one or more creatures attack you", "if your team controls another Warrior",
"half your starting life total", "no mana was spent to cast it", "that spell's mana value",
"with equal or lesser toughness", "dies or is put into exile", "the next spell you cast this
turn costs {1} less to cast", "~ is also a Cleric, Rogue, Warrior, and Wizard", "Target
opponent exiles a creature they control and their graveyard", "Sacrifice ~ and another
green creature", "with two +1/+1 counters and a lifelink counter on it", "from among cards
(you own) exiled with ~", "until you exile another card with ~", "If that spell would be put
into a graveyard, put it on the bottom of its owner's library instead" (play terms the
renderer used to drop), "that opponent", "the last chosen player/color", "spent on a
creature spell", "When a creature is put onto the battlefield this way", "the number of
cards revealed this way", token copies "tapped and attacking" in each wording, "you've cast
another/an instant or sorcery spell this turn" (a spell's own cost, CR 601.2i). New
equivalences: "each of your opponents" = "each opponent" and, separately, "one or more of
your opponents" = "one or more opponents" (CR 102.2, 102.3; the two are never equated),
the defending player named again as "their"/"they", "a land or legendary Turtle card".
Two existing comparison rules were fixed: the "exiled with ~ with mana value" reordering
matched across `{alt:...}` alternatives, and "Choose target ... . Copy that spell or
ability" left "or ability" behind.

Review: every play term `with_play_terms` and `static_permission_terms` don't word (a
cast or land rider, cost reduction or increase per card, a free option, the cast face, "from
any graveyard", spending a mana type or snow mana as any color, "put it ... instead" for a
static permission) is now a gap instead of being dropped (no passing card changed; it
exposes e.g. The Great Work's "from any graveyard", worded "your graveyard" before).
Reviewed and kept: the defending-player pronoun rules (every card with "defending player
... their/they control" in one sentence means defending player), "a land or legendary
Turtle card" (the general "[type] card or [qualified] card" rule; Lara Croft's "legendary
artifact card or legendary land card" reads the same way), Animal Attendant's duration (the
replacement is locked to the spell object, which a later object never is, CR 400.7), and
Aven Warcraft's protection outside the threshold "if" (without a chosen color "the chosen
color" matches nothing, so no protection is granted).

## Parser bugs found

Each has an in-game test in `crates/mtg-engine/tests/cards/roundtrip_tail_6.rs` that fails
on the old code (checked by restoring the old parser files and running the test) and a row
in the bug table (`FINDINGS`):

- Celestial Dawn (and Biotransference, Arcane Adaptation, Maskwood Nexus, Dune Chanter,
  Encroaching Mycosynth, Roshan, Rukarumel, Conspiracy, Leyline of Transformation): "cards
  you own that aren't on the battlefield" left out the stack, so a spell you own that
  another player controls wasn't made white (CR 112.1). `statics_other_zones::
  NOT_ON_BATTLEFIELD` (used by `becomes_grammar` and `choice_grammar_statics` too).
- Deafening Silence, Phyrexian Censor, Ethersworn Canonist: the spell limit's filter kept
  the phrase parser's probe "card", so a cast copy of a card (Isochron Scepter) was neither
  stopped by the limit nor counted toward it (CR 112.1b, 707.12).
  `cast_restrictions_more::spells`.
- Stonework Packbeast, Tajuru Paragon, Veteran Adventurer: "~ is also a Cleric, Rogue,
  Warrior, and Wizard" applied only on the battlefield; their rulings say all zones (a CDA,
  CR 604.3). New `patterns/r604_is_also_subtypes_cda.rs`.
- Skeletonize: the delayed trigger's capture of the creating ability's references
  (`delayed_grammar::capture_refs`, and `triggers_delayed::capture`) also rewrote the
  token's own ability, so "{B}: Regenerate this token" regenerated Skeletonize. Abilities a
  delayed effect creates or grants are now left alone.
- Gollum, Scheming Guide: "Reveal that card" (the top card the guess was about) was read as
  a dig step choosing one of the two cards looked at, so you could reveal the other one.
  `dig_grammar` no longer reads "that card" as one of several cards dug, and
  `choice_grammar_hidden` reads it as the top card of your library.

Checked in play (the tests pass on the old code too): Gluttonous Hellkite counts every
creature sacrificed, tokens included; Madcap Experiment's damage is the number of cards
revealed; Cherished Hatchling's granted ability makes the Dinosaur fight.

Animal Attendant's "that creature enters with an additional +1/+1 counter" replacement
lasted until end of turn; it now lasts until it applies, as the other mana riders' do (not
observable: a spell cast with that mana resolves before the turn's cleanup ends).

## Remaining

- Reyhan, Last of the Abzan: "if it had one or more +1/+1 counters on it, you may put that
  many +1/+1 counters" (rendered "it has ... X +1/+1 counters, where X is the number of
  +1/+1 counters on it"; the same value, LKI).
- Kamahl's Druidic Vow, Nashi, Searcher in the Dark: "land and/or legendary permanent cards
  with mana value X or less", "legendary and/or enchantment cards" (the shared qualities
  said once); Nashi's "If you put no cards into your hand this way".
- Blex, Vexing Pest // Search for Blex: "put any number of them into your hand and the rest
  into your graveyard. You lose 3 life for each card you put into your hand this way".
- Memory Theft: "a card with an Adventure that player owns from exile" (quality order).
- Check for Traps: "they lose 1 life" for the target opponent named before (rendered "that
  player").
- The Enigma Jewel // Locus of Enlightenment: the craft materials ("nonlands with activated
  abilities"), as before.
- Crystalline Giant: "choose a kind of counter at random that ~ doesn't have on it from
  among ..." (a custom effect and a chain of "if that many is N").
