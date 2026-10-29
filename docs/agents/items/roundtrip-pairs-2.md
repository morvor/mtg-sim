# roundtrip-pairs-2: the 3+-card clusters after paper-scope-1, and the next 25 two-card clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 28, origin 52c17bfe, after
paper-scope-1 counted 1,061 more paper cards): 27,828 of 31,667 fully supported cards pass;
3,705 mismatch (939 single-card clusters, 242 two-card clusters and the clusters below).
Many of the newly counted cards (Masters Edition, Vintage Masters, Tempest Remastered
reprints: old cards with old wordings) are new to the round trip. Runs alone.

CLUSTERS TAKEN (from `roundtrip --clusters 1200` on 52c17bfe): every cluster of 3+ cards,
then the first 25 two-card clusters in the tool's order:
   24  gap: Filter  [Resplendent Marshal; Rendmaw, Creaking Nest; Oblivious Bookworm]
   13  gap: EventsThisTurn  [Angel of Destiny; Boxing Ring; Ramses, Assassin Lord]
    9  gap: Effect  [Divine Intervention; Clockwork Steed; Clockwork Avian]
    4  gap: Condition  [Reset; Zimone, All-Questioning; Hero of Bretagard]
    4  oracle `have` vs rendered `∅`  [Varchild's War-Riders; Flaring Flame-Kin; Shyft]
    4  oracle `without flying that is attacking` vs rendered `attacking you without flying`  [Giant Trap Door Spider; Snow Fortress; Hunting Kavu]
    3  gap: StaticEffect  [Heartwood Dryad; Wall of Diffusion; Ranar the Ever-Watchful]
    3  gap: Value  [Impact Resonance; Taste of Paradise; Explosion of Riches]
    3  gap: remembering Before  [Oversimplify; Faerie Slumber Party; Luminate Primordial]
    3  oracle `cast of the chosen type` vs rendered `of {alt:the chosen type|that type} cast`  [Stenn, Paranoid Partisan; Cloud Key; Umori, the Collector]
    3  oracle `creature token of the chosen` vs rendered `colorless chosen color chosen type`  [Volrath's Laboratory; Riptide Replicator; Sarpadian Empires, Vol. VII]
    3  oracle `it is blocking` vs rendered `blocked by ~it`  [Goblin Snowman; Wall of Vipers; Tinder Wall]
    3  oracle `nonbasic` vs rendered `not basic`  [Choking Sands; Helldozer; Molten Rain]
    3  oracle `that player exile a card` vs rendered `exile a card in {alt:that player's|its}`  [Fiend of the Shadows; Elkin Lair; Kheru Mind-Eater]
    3  oracle `that player exile card from` vs rendered `{alt:that player|it} exile card from the`  [Dazzling Sphinx; Black Widow, Super Spy; Gríma, Saruman's Footman]
    3  oracle `where x is the number` vs rendered `∅`  [Thief of Blood; Hellfire; Thromok the Insatiable]
    3  oracle `would be put to owner` vs rendered `you own would be put`  [Yawgmoth's Will; Gaea's Will; Necrodominance]
    3  oracle `you own exiled with ~` vs rendered `exiled with ~it you own`  [Gustha's Scepter; Bottled Cloister; Endless Horizons]
    3  oracle `∅` vs rendered `a permanent {opt:that is}`  [Dream Prowler; Security Bypass; Yuan-Ti Malison]
    2  gap: qualities of an alternative list  [Soldevi Adnate; Desdemona, Freedom's Edge]
    2  gap: remembering TopOfLibrary  [Parker Luck; Keen Duelist]
    2  gap: several remembered numbers  [Hawkeye, Master Marksman; Tranquil Frillback]
    2  oracle `1 of it to owner` vs rendered `a card from among it`  [Browse; Flow State]
    2  oracle `1 of it` vs rendered `a card drawn this way`  [Krovikan Sorcerer; Soldevi Sage]
    2  oracle `1 or more +1/+1 counter` vs rendered `x +1/+1 counter from among`  [Jetfire, Ingenious Scientist // Jetfire, Air Guardian; Ooze Flux]
    2  oracle `1 or more +1/+1 counter` vs rendered `x +1/+1 counter from ~`  [Simic Manipulator; Arcee, Sharpshooter // Arcee, Acrobatic Coupe]
    2  oracle `1 or more {e} if` vs rendered `{alt:any amount of|one or more} {e} if you do`  [Pia Nalaar, Chief Mechanic; Localized Destruction]
    2  oracle `1 or more` vs rendered `any number of`  [Loamcrafter Faun; Specter of Mortality]
    2  oracle `a +1/+1 counter on ~it` vs rendered `x +1/+1 counter on it`  [Michelangelo, On the Scene; Batroc the Leaper]
    2  oracle `a card name exile the` vs rendered `{alt:a|any} card name exile the`  [Demonic Consultation; Divining Witch]
    2  oracle `a card name reveal a` vs rendered `{alt:a|any} card name reveal a`  [Cursed Scroll; Magus of the Scroll]
    2  oracle `a card name that player` vs rendered `{alt:a|any} card name reveal the`  [Vexing Arcanix; Petra Sphinx]
    2  oracle `a opponent control deal combat` vs rendered `your opponent control deal combat`  [Nelly Borca, Impulsive Accuser; Norn's Decree]
    2  oracle `a opponent control deal x` vs rendered `{alt:an opponent controls|your opponents control} deal x damage to`  [Dong Zhou, the Tyrant; Alpha Brawl]
    2  oracle `a opponent control until ~` vs rendered `{alt:an opponent controls|your opponents control} until ~ leave the`  [Turncoat Kunoichi; Driftgloom Coyote]
    2  oracle `a opponent control would die` vs rendered `{alt:an opponent controls|your opponents control} would die exile it`  [Ravenloft Adventurer; Ravenous Slime]
    2  oracle `any player may activate this` vs rendered `activate only during any upkeep`  [Armageddon Clock; Infinite Hourglass]
    2  oracle `base power become equal to` vs rendered `have base power`  [Belligerent Yearling; PuPu UFO]
    2  oracle `block or crew vehicle it` vs rendered `or block {alt:enchanted permanent's|its} activated ability`  [Bound in Gold; Intercessor's Arrest]
    2  oracle `blocking or` vs rendered `{opt:that is} blocking ~ or each`  [Greater Werewolf; Kjeldoran Frostbeast]
    2  oracle `card name other than a` vs rendered `not basic land card name`  [Desperate Research; Necromentia]
    2  oracle `card` vs rendered `∅`  [Garruk's Harbinger; Sanctum of Shattered Heights]
    2  oracle `cast any number of the` vs rendered `may cast it`  [Bloodthirsty Adversary; The Tale of Tamiyo]
    2  oracle `cast of the chosen type` vs rendered `of {alt:the chosen type|that type} cast cost {1}`  [Gathering Stone; Herald's Horn]

Note: clusters where the rendered side is `∅` for content the Oracle text has (e.g. "where
x is the number" on Thief of Blood, Hellfire, Thromok) may mean the compiled ability lost
that content — a parser bug, not a renderer gap. Check each.

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects, zero-quantity
choices?) with a fix + in-game test in `tests/cards/roundtrip_pairs_2.rs` that fails on the
old code + a row in the bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else
renderer wording (independently of the parser), or a CR-justified equivalence. Add "Parser
bugs found" and "Remaining" sections. Regenerate docs/ROUNDTRIP.md and
docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Result

27,828 of 31,667 fully supported cards passed at the start (52c17bfe); 27,921 of 31,711
pass after merging origin (origin's passing list at the merge has 27,828 cards). 69 of the
147 cards in the taken clusters pass.

Renderer (small hooks in the existing files and one new module, `render/graveyard_order.rs`):
"Hellfire deals X plus 3 damage to you, where X is ..." (a sum's X said with the amount) and
"creatures that died this way"; "you may have target creature get +X/+X ..." and "you may
have ~ become the color or colors of your choice" (a value kept or a choice made first);
a cumulative upkeep cost another player performs ("Have an opponent create ...");
"has trample, and has "{R}: ..."" (`{opt:has}` before the quoted abilities); "target
creature without flying that's attacking you"; "Spells you cast of the chosen type";
"a creature token of the chosen color and type" (the placeholder subtypes no longer printed
raw); "target creature it's blocking" (CR 509.1g) and "each creature blocking or blocked by
~"; "if it's nonbasic" (CR 205.4c); "If a card would be put into your graveyard from
anywhere this turn" (a card goes to its owner's graveyard, CR 400.3; the duration said with
the event); "a card you own exiled with ~"; "as long as it's attacking alone"; "Put one of
them into your hand", "exile the rest" (CR 701.13a), "discard one of them" (one of the cards
drawn); "As this creature enters, remove all counters from all permanents. ~ enters with ...
for each counter removed this way"; "Devour X, where X is the number of creatures devoured
this way"; "if it's a card of the chosen type", "if that card has the chosen name"; "a card
name other than a basic land card name"; this turn's events ("that attacked you", "dealt
damage this turn by a source you controlled", "if it's the first time that creature has
become tapped this turn", "a creature that fought this turn", "each player who tapped a
land for mana this turn", "an oil counter was removed from a permanent you controlled",
"~ blocked or was blocked by a blue creature", "the number of creatures that left the
battlefield under your control this turn"); "each player who sacrificed a Plains this way"
(CR 701.21a); "base power becomes equal to ..." (CR 613.4b); "Any player may activate this
ability but only during any upkeep step"; the Clockwork creatures' "Put up to X +1/+0
counters on ~. This ability can't cause the total number ... to be greater than four";
"can block creatures with shadow as though it had shadow"; "The first card you foretell
each turn costs {0} to foretell"; "the greatest amount of damage dealt by a source to a
permanent or player this turn"; "that player exiles a card from their hand [face down]";
"collect evidence X", "unlock a locked door of a Room you control", "you rolled a 6 this
turn", "you control a prime number of lands"; "if ~ is in your graveyard with a creature
card directly above it", "three or more creature cards are above ~" (CR 404.2).

## Parser bugs found

Tests in `crates/mtg-engine/tests/cards/roundtrip_pairs_2.rs`; both fail on the old code
(checked by restoring the old parser code and running the tests):

- Hellfire: "~ deals X plus 3 damage to you, where X is the number of creatures that died
  this way" kept the spell's own X (0) in "X plus 3", so it dealt 3 damage whatever died.
  `patterns/damage_removal.rs` (`amount_phrase` puts the defined X wherever the amount uses
  it).
- Norn's Decree: "Whenever one or more creatures an opponent controls deal combat damage to
  you, that opponent gets a poison counter": "that opponent" was the player dealt damage, so
  you got the poison counter. `patterns/triggers.rs` (the controller of those creatures).
  With creatures of several opponents dealing damage at once it triggers once and each of
  those opponents gets a counter.

Checked in play (tests in the same file, passing on the old code too): Belligerent Yearling
keeps the power it copied when the Dinosaur is pumped later (CR 608.2h); Thief of Blood
gets a counter for each counter of any kind on any permanent; Desolation deals its damage to
the player who sacrificed a Plains another player owns. Also checked by reading the AST:
Volrath's Laboratory's token gets the chosen color (`tokens.rs`), Krovikan Sorcerer discards
one of the two cards drawn, Necromentia counts the cards found in the hand, Bloodthirsty
Adversary asks for each copy, Oversimplify counts each player's own exiled creatures,
Vexing Arcanix deals no damage when the library is empty (no card revealed).

## Remaining

Of the 147 cards in the taken clusters, these still mismatch; each was checked against the
AST and plays as the text says (wording only) unless noted:

- The 24-card `gap: Filter` cluster's custom qualities not yet worded: Resplendent Marshal,
  Cemetery Desecrator, Mechtitan Core ("the source or the object it became"); Rendmaw ("two
  or more card types"); Oblivious Bookworm ("entered face down"); Emrakul, the World Anew
  ("permanents that were cast this turn"; also "Madness—Pay six {C}"); Everything Comes to
  Dust (convoked it); Sentinel Tower ("cast before it this turn"); Full Bore ("cast for its
  warp cost"); Alania; Satoru; Library of Leng ("an effect causes you to discard"); Prowl
  and Voltaic Visionary ("play a card exiled with ~" as land-or-spell); Zenos yae Galvus,
  Diabolic Servitude, Havengul Laboratory ("the creature put onto the battlefield with ~");
  C.A.M.P. ("shares a color with the mana that land produced" for a target); Zimone's
  Hypothesis; The Enigma Jewel (as before).
- `EventsThisTurn` cards whose clause isn't worded: Angel of Destiny ("each player ~ attacked
  this turn" inside a per-player condition), Boxing Ring ("with the same mana value"),
  Shelob (the copy's exceptions), Churning Reservoir ("a permanent with an oil counter on it
  was put into a graveyard"), Avatar Aang ("if you've done all four this turn").
- Custom effects and conditions: Divine Intervention ("When you remove the last
  intervention counter"), Crystalline Giant (as before), Word of Command (hand-written
  steps), Reset ("after their upkeep step" in a casting restriction), Hero of Bretagard and
  Ranar ("or a spell or ability you control exiles one or more permanents"), Taste of
  Paradise ("for each additional {1}{G} you paid"), Explosion of Riches ("Whenever a card
  is drawn this way").
- Remembered groups and numbers: Oversimplify, Faerie Slumber Party, Luminate Primordial
  (`remembering Before`), Parker Luck, Keen Duelist (`remembering TopOfLibrary`), Hawkeye,
  Tranquil Frillback (`several remembered numbers`).
- Wording: Zimone, All-Questioning ("put that many +1/+1 counters on it" for the prime
  count); Eelectrocute (the condition's place); Elkin Lair ("exiles a card at random from
  their hand"); Kheru Mind-Eater (two sentences of one permission); Dazzling Sphinx, Gríma
  ("the exiled cards that weren't cast this way"); Black Widow; Soldevi Adnate, Desdemona
  (`qualities of an alternative list`); Flow State ("an instant card and a sorcery card in
  your graveyard"); Vexing Arcanix, Petra Sphinx ("If it doesn't" / the revealed card
  existing); Nelly Borca, Norn's Decree ("creatures an opponent controls", "the controller
  of those creatures", "one or more players being attacked are poisoned"); Bound in Gold,
  Intercessor's Arrest (three restrictions on one line); Desperate Research ("each of them
  with that name"), Necromentia ("for each card exiled from their hand this way"); Garruk's
  Harbinger, Sanctum of Shattered Heights ("a land card or Shrine card"); Bloodthirsty
  Adversary, The Tale of Tamiyo ("cast any number of the copies"); Michelangelo, Batroc
  ("a +1/+1 counter on him for each ...": the pronoun).
- As in roundtrip-pairs-1: Jetfire, Ooze Flux, Simic Manipulator, Arcee; Pia Nalaar,
  Localized Destruction; Loamcrafter Faun, Specter of Mortality; Dong Zhou, Alpha Brawl;
  Turncoat Kunoichi, Driftgloom Coyote, Ravenloft Adventurer, Ravenous Slime.
