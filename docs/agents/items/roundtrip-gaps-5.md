# roundtrip-gaps-5: the remaining gap clusters (custom nodes) and the next 40 two-card clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 29, origin 6026924c, after
roundtrip-pairs-2 landed): 27,925 of 31,789 fully supported cards pass; 3,730 mismatch (953
single-card clusters, 240 two-card clusters and the clusters below). Runs alone.

Most remaining `gap:` cards are abilities compiled to named custom nodes (custom filter
qualities, custom conditions/effects/values implemented in code: Divine Intervention, Reset,
Word of Command, Crystalline Giant, "the source or the object it became", "the creature put
onto the battlefield with ~") that earlier items left unworded. The job: give every custom
node these cards use a wording derived from what the node does (read its implementation and
its tests, not the card's text), so the round trip actually checks these cards. Where a
custom node's implementation doesn't do what the card's text says, that's a bug: fix it with
an in-game test.

CLUSTERS TAKEN (from `roundtrip --clusters 1200` on 6026924c): every `gap:` cluster, the
3+-card wording cluster, and the first 40 two-card clusters in the tool's order:
   29  gap: Filter  [Phyrexian Grimoire; Resplendent Marshal; Rendmaw, Creaking Nest]
   13  gap: StaticEffect  [Paliano Vanguard; Animus of Predation; Aether Searcher]
    9  gap: Effect  [Divine Intervention; Frankenstein's Monster; Spikeshell Harrier]
    5  gap: Condition  [Reset; Hero of Bretagard; Ranar the Ever-Watchful]
    4  gap: Value  [Central Elevator // Promising Stairs; Taste of Paradise; Ellywick Tumblestrum]
    3  gap: remembering Before  [Oversimplify; Faerie Slumber Party; Luminate Primordial]
    3  oracle `that player exile card from` vs rendered `{alt:that player|it} exile card from the`  [Dazzling Sphinx; Black Widow, Super Spy; Gríma, Saruman's Footman]
    2  gap: EventsThisTurn  [Overclocked Electromancer; Avatar Aang // Aang, Master of Elements]
    2  gap: a remembered group no instruction names  [Storm Herald; Coin of Fate]
    2  gap: qualities of an alternative list  [Soldevi Adnate; Desdemona, Freedom's Edge]
    2  gap: remembering TopOfLibrary  [Parker Luck; Keen Duelist]
    2  gap: several remembered numbers  [Hawkeye, Master Marksman; Tranquil Frillback]
    2  oracle `1 or more +1/+1 counter` vs rendered `x +1/+1 counter from among`  [Jetfire, Ingenious Scientist // Jetfire, Air Guardian; Ooze Flux]
    2  oracle `1 or more +1/+1 counter` vs rendered `x +1/+1 counter from ~`  [Simic Manipulator; Arcee, Sharpshooter // Arcee, Acrobatic Coupe]
    2  oracle `1 or more {e} if` vs rendered `{alt:any amount of|one or more} {e} if you do`  [Pia Nalaar, Chief Mechanic; Localized Destruction]
    2  oracle `1 or more` vs rendered `any number of`  [Loamcrafter Faun; Specter of Mortality]
    2  oracle `a +1/+1 counter on ~it` vs rendered `x +1/+1 counter on it`  [Michelangelo, On the Scene; Batroc the Leaper]
    2  oracle `a artifact a creature a` vs rendered `from among the nonland permanent`  [Cataclysmic Gearhulk; Mythos of Snapdax]
    2  oracle `a card name that player` vs rendered `{alt:a|any} card name reveal the`  [Vexing Arcanix; Petra Sphinx]
    2  oracle `a opponent control deal combat` vs rendered `your opponent control deal combat`  [Nelly Borca, Impulsive Accuser; Norn's Decree]
    2  oracle `a opponent control deal x` vs rendered `{alt:an opponent controls|your opponents control} deal x damage to`  [Dong Zhou, the Tyrant; Alpha Brawl]
    2  oracle `a opponent control until ~` vs rendered `{alt:an opponent controls|your opponents control} until ~ leave the`  [Turncoat Kunoichi; Driftgloom Coyote]
    2  oracle `a opponent control would die` vs rendered `{alt:an opponent controls|your opponents control} would die exile it`  [Ravenloft Adventurer; Ravenous Slime]
    2  oracle `block or crew vehicle it` vs rendered `or block {alt:enchanted permanent's|its} activated ability`  [Bound in Gold; Intercessor's Arrest]
    2  oracle `card` vs rendered `∅`  [Garruk's Harbinger; Sanctum of Shattered Heights]
    2  oracle `cast any number of the` vs rendered `may cast it`  [Bloodthirsty Adversary; The Tale of Tamiyo]
    2  oracle `choose a color of a` vs rendered `add 1 mana of any`  [Corrupted Grafstone; Meteor Crater]
    2  oracle `choose a nonland card from` vs rendered `{opt:you} choose a nonland card`  [Memory Theft; Check for Traps]
    2  oracle `choose a opponent that player` vs rendered `a opponent`  [Skullwinder; Goblin Festival]
    2  oracle `choose any number of target` vs rendered `any number of target creature`  [Cauldron Haze; Cauldron of Souls]
    2  oracle `choose any target reveal card` vs rendered `reveal card from the top`  [Explosive Revelation; Erratic Explosion]
    2  oracle `choose target creature a opponent` vs rendered `roll a d20 1 9`  [Power of Persuasion; Scion of Stygia]
    2  oracle `choose target creature you control` vs rendered `if you control 3 or`  [Blizzard Brawl; Duel for Dominance]
    2  oracle `choose target creature you control` vs rendered `target creature you control deal`  [Grim Contest; Nibelheim Aflame]
    2  oracle `choose up to 1 target` vs rendered `~ deal x damage to`  [Mega Flare; Shellshock]
    2  oracle `choose` vs rendered `∅`  [Coax from the Blind Eternities; Karn, the Great Creator]
    2  oracle `chosen creature` vs rendered `last chosen card`  [Metamorphic Alteration; Dauntless Bodyguard]
    2  oracle `colorless mana` vs rendered `{c}`  [Pale Moon; Ritual of Subdual]
    2  oracle `combat on a opponent turn` vs rendered `a opponent turn only during`  [Take the Bait; Wake the Dead]
    2  oracle `combat on your turn` vs rendered `your turn only during combat`  [Savage Beating; Melee]
    2  oracle `commander` vs rendered `∅`  [Flamekin Herald; Yuriko, the Tiger's Shadow]
    2  oracle `copy it this turn` vs rendered `this turn copy it`  [The Mirari Conjecture; Jace Reawakened]
    2  oracle `cost pay cost {1} less` vs rendered `ability you activate cost {1}`  [Henzie "Toolbox" Torre; Auriok Steelshaper]
    2  oracle `cost pay cost {2} less` vs rendered `ability you activate cost {2}`  [Warbringer; Catalyst Stone]
    2  oracle `cost {1} less to cast` vs rendered `have ~ cost {x} less`  [Mutated Cultist; Elminster]
    2  oracle `create a food token or` vs rendered `{alt:as this resolves, choose one —   create a food token.   create a treasure token.|create a {opt:your choice of} food or treasure token}`  [Ant-Man's Army; Tireless Provisioner]
    2  oracle `created` vs rendered `exiled`  [Dual Nature; Saproling Burst]

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects, zero-quantity
choices?) with a fix + in-game test in `tests/cards/roundtrip_gaps_5.rs` that fails on the
old code + a row in the bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else
renderer wording (independently of the parser), or a CR-justified equivalence. Judgment
calls go in the interpretation register (CLAUDE.md). Add "Parser bugs found" and "Remaining"
sections. Regenerate docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check`
must pass.

## Result

27,925 of 31,789 fully supported cards passed at the start (6026924c); 28,016 of 31,789 pass
on the branch before the merge, and 28,040 of 31,851 after merging the later landings
(origin's passing list at that merge has 27,924 cards). 73 of the 148 cards in the taken
clusters pass.

Renderer (new modules `render/draft_terms.rs`, `render/custom_nodes.rs`,
`render/linked_terms.rs`; small hooks in the existing dispatch chains): the Conspiracy draft
abilities, each from what `draft.rs` does ("Draft ~ face up", "As you draft a creature
card, you may reveal it, note its name, then turn ~ face down", "you guessed correctly for
a card named ~", "the number of cards you removed from the draft with cards named ~",
Animus of Predation's per-keyword statics as one "If you removed a creature card with
flying ... The same is true for ..." line, "a player you noted for cards named ~"); the
object a linked ability noted, named by what that ability did ("the chosen creature", "the
last chosen card" for an activated choice, "the creature put onto the battlefield with ~",
CR 607.2c/d; `FaceInfo::noted`); "tokens created with ~" (a linked token was created by
the source, CR 607.2c); "the top two cards of your graveyard" / "one of the top two
cards"; "another creature card" for the source-or-what-it-became quality (CR 400.7);
"creatures other than ~ and the chosen creature"; "with two or more card types"; "that
were cast this turn"; "you exiled" / "cards they exiled with ~"; "that shares a creature
type with a creature that convoked ~" and "except those that ..."; "if that creature was
cast for its warp cost"; "that shares a color with the mana that land produced" as a
condition; "Whenever ... play a card exiled with ~" and "play a card with two or more card
types" (cast or land played, also in a list of events); "When you remove the last
intervention counter from ~", "the game is a draw" (CR 104.4a); "a spell or ability you
control exiles one or more permanents from the battlefield"; "When the chosen player loses
the game"; "Whenever an instant or sorcery spell is cast during your turn"; "the number of
instant and sorcery spells cast before that spell this turn"; "Cast ~ only during an
opponent's turn after their upkeep step" (CR 503.2) and "only during combat on your turn";
"any of those results was 10 or higher"; "different names among unlocked doors of Rooms
you control"; "differently named dungeons you've completed"; "that excess damage"; "if
you've done all four this turn"; "If that opponent's speed is greater than each other
player's speed, reduce that opponent's speed by 1. This effect can't reduce their speed
below 1."; "you may pay {G} up to three times. When you pay this cost one or more times,
choose up to that many"; "Whenever a card is drawn this way"; "target opponent chosen at
random from among your opponents"; "Copy them. You may cast any number of the copies";
"you choose the permanents for each player instead" (`apnap::KEEP_CHOOSER`); "Return any
number of Aura cards ... attached to creatures you control"; "choose a creature you control
it could be attached to. If you do, it enters attached to that creature"; "put ~ into its
owner's graveyard instead of onto the battlefield"; Word of Command's four steps; "attach
~ to another one of your opponents chosen at random"; "a creature or player" chosen on
resolution; Crystalline Giant's "choose a kind of counter at random that ~ doesn't have on
it from among ...", "copy all spells you control, then copy all other activated and
triggered abilities you control"; "The escape cost is equal to its mana cost plus ..."
(the renderer used to drop the mana cost: "escape—Exile three other cards"); "Commander
spells", "Commander ninjutsu", "Madness—Pay six {C}", "Dash costs you pay cost {2} less",
"Choose a color of a permanent you control. Add one mana of that color.", "colorless mana"
(CR 107.4c), "Choose an opponent. That player ...", "a creature card or Garruk planeswalker
card", "chooses an artifact, ... from among the nonland permanents they control", "Until
end of turn, whenever you cast a spell, copy it", "~ has five or more counters on it",
"Choose any number of target creatures. Each of those creatures ...", "Create a Food token
or a Treasure token", "each player may play lands and cast spells from among ...", "to
that player" for the controller of a target an opponent controls.

## Parser bugs found

Tests in `crates/mtg-engine/tests/cards/roundtrip_gaps_5.rs`; both fail on the old code
(checked by restoring the old engine file and running the test):

- Bloodthirsty Adversary: "When you pay this cost one or more times, ... exile up to that
  many target instant and/or sorcery cards ... and copy them. You may cast any number of the
  copies without paying their mana costs." The last sentence was left in the triggered
  ability, which offered the casting before the reflexive ability made the copies, so they
  were never cast (CR 603.12). `patterns/reflexive_continues.rs`: a sentence after "When
  you pay this cost one or more times" continues that reflexive ability, as after "When you
  do".
- Word of Command: "only if mana they produce is spent to activate other mana abilities of
  lands the player controls and/or to play that card" wasn't implemented: mana left in the
  pool after the card was played could pay for anything. `player_control.rs`
  (`restrict_lands_only_mana`, called from `Game::add_mana`): a spending restriction on that
  mana (CR 106.6); provisional register entry `word-of-command-mana` (the restriction
  allows any ability of a land, not only mana abilities).

Checked in play (tests in the same file, passing on the old code too): Spikeshell Harrier
slows the opponent who controlled the creature when another player owns it (the
controller as it last existed, CR 608.2h); Faerie Slumber Party counts the opponents who
controlled the returned creatures, not their owners; Oversimplify gives each player's
Fractal the power of the creatures they controlled. Also checked by reading the AST or the
existing tests: Mythos of Snapdax's "you choose the permanents for each player" (the
chooser variable), Kaboom!, Explosion of Riches, Cass, Alpha Brawl, Grim Contest, Keen
Duelist and Parker Luck (each loses life for the other's card), Vexing Arcanix, Desdemona
(escape's mana cost is added, `kw/escape.rs`), Satoru, the Infiltrator.

## Remaining

Of the 148 cards in the taken clusters, these still mismatch; each was checked against the
AST and plays as the text says (wording only) unless noted:

- Word of Command: the restriction allows abilities of lands the player controls, not only
  their mana abilities (see the register entry), so it's worded that way.
- "Choose target ..." first, then the instructions about the chosen targets (the general
  form isn't rendered; it's offered only for one instruction about several targets):
  Blizzard Brawl, Duel for Dominance, Grim Contest ("each of those creatures deals damage
  ... to the other"), Nibelheim Aflame, Power of Persuasion, Scion of Stygia, Delina,
  Explosive Revelation, Erratic Explosion ("that permanent or player"), Mega Flare,
  Shellshock ("each of the chosen creatures"), Kaboom!, Turncoat Kunoichi ("the chosen
  creature").
- "Remove one or more +1/+1 counters" compiled as X counters with "X can't be 0", and the
  number as X: Jetfire, Ooze Flux, Simic Manipulator, Arcee; Pia Nalaar, Localized
  Destruction ("the amount of {E} paid this way"); Loamcrafter Faun, Specter of Mortality
  ("one or more" compiled as any number: none chosen does nothing either way).
- Remembered groups and numbers: Oversimplify, Faerie Slumber Party, Luminate Primordial
  (`remembering Before`), Parker Luck, Keen Duelist (`remembering TopOfLibrary`), Dazzling
  Sphinx and Gríma ("the exiled cards that weren't cast this way"), Storm Herald ("those
  Auras"), Vault 21 ("for each of those cards").
- Statics and replacements in one sentence or another order: Bound in Gold, Intercessor's
  Arrest (three restrictions), Noble Banneret ("~ and those creatures"), Ravenloft
  Adventurer, Ravenous Slime ("instead exile it and put ..."), Library of Leng ("discard
  it, but you may put it on top of your library instead"), Uba Mask ("exiles that card face
  up"), Shared Fate ("one of their opponents' libraries"; two abilities in one sentence),
  Michelangelo, Batroc ("a +1/+1 counter on him for each"), The Enigma Jewel (the craft
  materials), Cass ("Equipment that were attached to it"), Desdemona (the duration's place
  and the alternative list), Soldevi Adnate (`qualities of an alternative list`), Henzie
  (the comparison of "Blitz costs you pay cost {1} less" inside an alternative).
- Other wordings: Frankenstein's Monster ("If you can't, ..."), Comet (the loyalty symbols
  in the die table), Zenos ("creatures other than ~ and the chosen creature" after
  "Until end of turn"), Alania ("the first Otter spell other than ~"), Satoru ("none of
  them were cast"), Oblivious Bookworm ("a permanent entered the battlefield face down
  under your control"), Evelyn, Haldan ("cards you exiled that have fetch counters on
  them"), Aether Searcher ("a name noted as you drafted cards named ~"), Central Elevator
  ("a Room you control" for a library card), Havengul Laboratory ("if you sacrificed three
  or more Clues this turn"), Resplendent Marshal, Mechtitan Core ("other than ~" / "except
  ~"), Memory Theft, Check for Traps, Vexing Arcanix, Petra Sphinx, Coax, Karn ("or choose
  a face-up ... card"), Nelly Borca, Norn's Decree, Dong Zhou's cluster partner Alpha Brawl
  ("each of those creatures deals damage equal to its power"), False Orders, Balduvian
  Warlord, Driftgloom Coyote ("If that creature had power 2 or less"), Black Widow ("the
  exiled nonland card"), Mutated Cultist, Elminster ("{1} less ... for each"), Taste of
  Paradise ("an additional 3 life for each additional {1}{G} you paid"), Hero-like word
  orders.
