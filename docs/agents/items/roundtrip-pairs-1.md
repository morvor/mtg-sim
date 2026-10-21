# roundtrip-pairs-1: the last 3+-card clusters and the first 45 two-card clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 21/22, origin 48eb37a7, after
roundtrip-tail-6 landed): 26,756 of 30,413 fully supported cards pass; 3,527 mismatch, in
729 single-card clusters, 261 two-card clusters and the clusters below. Runs alone.

CLUSTERS TAKEN (from `roundtrip --clusters 1000` on 48eb37a7): every cluster of 3+ cards,
then the first 45 two-card clusters in the tool's order:
   11  gap: Filter  [Cathedral Membrane; Onakke Oathkeeper; Hollow Warrior]
   10  gap: a remembered group no instruction names  [Cache Grab; Bringer of the Last Gift; Sphinx's Tutelage]
    9  gap: permission term  [Durnan of the Yawning Portal; Chiss-Goria, Forge Tyrant; Mosswood Dreadknight // Dread Whispers]
    6  gap: Effect  [World at War; Portal Manipulator; Mishra, Claimed by Gix]
    6  gap: Value  [Lethal Exploit; Lifestream's Blessing; Jaws of Stone]
    6  gap: static permission term  [The Eighth Doctor; Urabrask // The Great Work; Risen Executioner]
    4  oracle `main phase there is a` vs rendered `turn it is your main`  [Moraug, Fury of Akoum; Grim Reaper's Sprint; All-Out Assault]
    3  oracle `it control with the greatest` vs rendered `with the {alt:greatest|highest} mana value`  [Riveteers Charm; End of the Hunt; Blot Out]
    3  oracle `may put any number of` vs rendered `{opt:may} put any number of`  [Kamahl's Druidic Vow; Nashi, Searcher in the Dark; Blex, Vexing Pest // Search for Blex]
    3  oracle `your library have no card` vs rendered `there is no card in`  [Fblthp, Impossibly Lost; Doctor Doom, Unrivaled; Stillness in Motion]
    2  gap: Condition  [Combat Celebrant; Graxiplon]
    2  gap: Modification  [The Eleventh Hour; Jace, Mirror Mage]
    2  gap: one-use replacement effect  [Mystic Reflection; Storyweave]
    2  oracle `1 or more +1/+1 counter` vs rendered `x +1/+1 counter from among`  [Jetfire, Ingenious Scientist // Jetfire, Air Guardian; Ooze Flux]
    2  oracle `1 or more +1/+1 counter` vs rendered `x +1/+1 counter from ~`  [Simic Manipulator; Arcee, Sharpshooter // Arcee, Acrobatic Coupe]
    2  oracle `1 or more {e} if` vs rendered `any amount of {e} if`  [Pia Nalaar, Chief Mechanic; Localized Destruction]
    2  oracle `1 or more` vs rendered `any number of`  [Loamcrafter Faun; Specter of Mortality]
    2  oracle `2 +1/+1 counter on ~` vs rendered `x +1/+1 counter on ~it`  [Orchard Elemental; Mind Maggots]
    2  oracle `2 or more equipment is` vs rendered `there is 2 or more`  [Brass Knuckles; Balan, Wandering Knight]
    2  oracle `2 target player each` vs rendered `target player draw a card`  [Huddle Up; Gleaming Splendor]
    2  oracle `4 or more damage was` vs rendered `the {opt:amount of} damage {opt:already} dealt`  [Burning-Eye Zubera; Rushing-Tide Zubera]
    2  oracle `a +1/+1 counter on ~it` vs rendered `x +1/+1 counter on it`  [Michelangelo, On the Scene; Batroc the Leaper]
    2  oracle `a artifact is destroyed this` vs rendered `it exist it is a`  [Acolyte Hybrid; Smashing Success]
    2  oracle `a card is exiled with` vs rendered `there is a card exiled`  [Smirking Spelljacker; Redemptor Dreadnought]
    2  oracle `a card name search target` vs rendered `{alt:a|any} card name search target`  [Unmoored Ego; The Stone Brain]
    2  oracle `a card type other than` vs rendered `artifact battle enchantment instant kindred`  [Stenn, Paranoid Partisan; Arachne, Psionic Weaver]
    2  oracle `a creature card a` vs rendered `up to 1 creature card`  [Michelangelo, Improviser; Yuna's Decision]
    2  oracle `a creature card is revealed` vs rendered `it is a creature card`  [Elven Farsight; Deceiver of Form]
    2  oracle `a creature is attacking you` vs rendered `there is a attacking creature`  [Heroic Return; Swat Away]
    2  oracle `a creature other than ~` vs rendered `other creature if you {alt:do not|can't}`  [Lord of the Pit; Liege of the Pit]
    2  oracle `a dragon was beheld` vs rendered `{alt:the behold cost was paid|you revealed a dragon card or chose a dragon as cast ~}`  [Osseous Exhale; Piercing Exhale]
    2  oracle `a enchanted or equipped creature` vs rendered `{alt:an enchanted creature or an equipped creature you control|an enchanted creature you control or an equipped creature you control}`  [Dogmeat, Ever Loyal; Reyav, Master Smith]
    2  oracle `a number of card from` vs rendered `the top x card of`  [Omarthis, Ghostfire Initiate; Eye of Yawgmoth]
    2  oracle `a number of time counter` vs rendered `x time counter on it`  [The Wedding of River Song; The Eleventh Doctor]
    2  oracle `a opponent control is dealt` vs rendered `{alt:an opponent controls|your opponents control} is dealt excess noncombat`  [Fall of Cair Andros; Toralf, God of Fury // Toralf's Hammer]
    2  oracle `a opponent control until ~` vs rendered `{alt:an opponent controls|your opponents control} until ~ leave the`  [Turncoat Kunoichi; Driftgloom Coyote]
    2  oracle `a opponent control where x` vs rendered `{alt:an opponent controls|your opponents control} where x is it`  [Foray of Orcs; Chocobo Kick]
    2  oracle `a opponent control would die` vs rendered `{alt:an opponent controls|your opponents control} would die exile it`  [Ravenloft Adventurer; Ravenous Slime]
    2  oracle `a opponent of your choice` vs rendered `each opponent`  [Abby, Merciless Soldier; Captive Audience]
    2  oracle `a player have 1 or` vs rendered `there is a player with`  [Naktamun Lorespinner // Wheel of Fortune; Aclazotz, Deepest Betrayal // Temple of the Dead]
    2  oracle `a target of target spell` vs rendered `the target of target {alt:spell or ability|spell, activated ability, or triggered ability}`  [Mizzium Meddler; Spellskite]
    2  oracle `ability of equipped creature is` vs rendered `player activate a ability that`  [Illusionist's Bracers; Battlemage's Bracers]
    2  oracle `amount of damage dealt to` vs rendered `{opt:amount of} damage dealt to ~it`  [Grothama, All-Devouring; Blazing Effigy]
    2  oracle `any number of time` vs rendered `∅`  [Kindle the Carnage; Ad Nauseam]
    2  oracle `any of that mana is` vs rendered `that mana is spent {alt:to cast|on}`  [Arena of Glory; Generator Servant]
    2  oracle `any opponent may have it` vs rendered `each opponent may ~it deal`  [Vexing Devil; Longhorn Firebeast]
    2  oracle `any opponent may have put` vs rendered `starting with the next opponent`  [Distant Memories; Sin Prodder]
    2  oracle `any player may activate this` vs rendered `activate only if it is`  [Mana Cache; Well of Knowledge]
    2  oracle `artifact named crown of empire` vs rendered `a artifact named crown of`  [Throne of Empires; Scepter of Empires]
    2  oracle `artifact spell` vs rendered `{alt:and|or} artifact`  [Renari, Merchant of Marvels; Mai and Zuko]
    2  oracle `as cast ~ may choose` vs rendered `{opt:as cast ~} may choose both instead`  [Nexus Mentality; Szat's Will]
    2  oracle `as long as a card` vs rendered `first strike double strike deathtouch`  [Eater of Virtue; Urborg Scavengers]
    2  oracle `as long as a exiled` vs rendered `first strike double strike deathtouch`  [Altar of the Wretched // Wretched Bonemass; Rayami, First of the Fallen]
    2  oracle `attach it to a creature` vs rendered `{opt:under your control} shuffle attach it to`  [Quest for the Holy Relic; Stonehewer Giant]
    2  oracle `attacking or blocking` vs rendered `a attacking creature or a`  [Sawback Manticore; Intrepid Ace]

Kamahl's Druidic Vow / Nashi / Blex and other cards earlier items listed as "Remaining
(wording only)" were triaged before: the job there is renderer wording (re-check anything the
earlier notes didn't cover). The `permission term` / `static permission term` gaps are new
(the roundtrip-tail-6 reviewer made the renderer report terms it used to drop silently):
word each term from the AST, and check the card really has that term (e.g. The Great Work's
"from any graveyard" vs the AST).

For each card: triage — parser bug first (does the AST mean what the text says, in
multiplayer, with control changes, tokens, copies, replacement effects, zero-quantity
choices?) with a fix + in-game test in `tests/cards/roundtrip_pairs_1.rs` that fails on the
old code + a row in the bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else
renderer wording (independently of the parser), or a CR-justified equivalence. Add "Parser
bugs found" and "Remaining" sections. Regenerate docs/ROUNDTRIP.md and
docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Result

26,756 of 30,413 fully supported cards passed at the start (48eb37a7); 26,901 of 30,413
pass on the branch before the first merge, and 26,980 of 30,643 after merging the later
landings (origin's passing list at the last merge has 26,756 cards). 115 of the 151 cards in
the taken clusters pass.

Renderer (small hooks in the existing files; no new module): the custom qualities of the
`gap: Filter` cluster ("creature it blocked this combat", "not declared as an attacking or
blocking creature this combat", "with base power 0", "that shares a creature type with each
creature tapped this way", "that are still exiled", "that player controls" for the player
a creature attacks, "another permanent card" other than the sacrificed one, landwalk of
each land type of the sacrificed land, "Creatures can't attack planeswalkers you control
unless ..."); conditions about what an instruction did ("If you control a Squirrel or
returned a Squirrel card to your hand this way", "If four or more nontoken permanents you
control were returned this way", "If no creature cards were revealed this way", "two
nonland cards that share a color were milled", "If a creature is put into exile this way,
return each other card exiled with ~", "If an artifact is destroyed this way", "creature
cards that weren't put there this way"); play-permission terms ("If you do, it has
affinity for artifacts", "That spell has undaunted", "If you do, it enters tapped" for a
played land or a cast permanent, "Spells you cast this way cost {2} less to cast", "You may
cast Equipment spells this way without paying their mana costs", "as an Adventure",
"spend colorless mana as though it were mana of any color", "from any graveyard" / "from
that player's graveyard", "if you pay {1} more to cast it for each ...", "mana from snow
sources", the same "If you do" for a played land and a cast spell); values taken "as you
cast ~", "total toxic value", "the discarded card's mana value", "the amassed Army's
power", "that excess damage" / "the excess"; additional phases "after the second main
phase this turn", "At the beginning of that combat, ...", "Those creatures are now
attacking that player", "you may reselect which player ~ is attacking" and "Only the player
~ is attacking may activate this ability and only during the declare attackers step",
Mishra's meld of an attacking pair; "Any opponent may have it deal 4 damage to them. If a
player does, ..." and "Any opponent may have you put that card into your graveyard. If a
player does, ... Otherwise, ..."; "Any player may activate this ability but only during
their draw step"; "the next time one or more creatures enter this turn, they enter as
copies of the chosen creature / each enters with two additional +1/+1 counters";
"enters under the control of an opponent of your choice"; "it's your main phase" (and a
main phase that isn't said to be yours is "a main phase": the condition alone is any
player's); "If your library has no cards in it"; "a creature is attacking you", "a card is
exiled with it", "a player has one or fewer cards in hand", "two or more Equipment are
attached to it", "4 or more damage was dealt to it this turn", "artifacts named Crown of
Empires and Scepter of Empires"; "a creature or planeswalker they control with the greatest
mana value among creatures and planeswalkers they control"; "a creature card and/or a land
card from your hand"; "Two target players each draw a card"; "If a Hero enters this way, it
enters with two additional +1/+1 counters on it"; "Dragon spells and artifact spells";
"an enchanted or equipped creature you control"; "Whenever an ability of equipped creature
is activated, if it isn't a mana ability"; "change a target of target spell or ability";
"has flying as long as a card exiled with it has flying. The same is true for ...";
"put two +1/+1 counters on ~ for each sprout vote"; "a card type other than creature or
land"; "any target other than that permanent"; "Search ..., put it onto the battlefield,
attach it to a creature you control, then shuffle"; "exile it instead of putting it
anywhere else" (a replacement for leaving the battlefield); "You may repeat this process any
number of times"; "If a Dragon was beheld"; "you may pay one or more {E}" (paying none is
not paying, as the parser records it).

Two renderer catch-alls that dropped qualities were removed: a becoming effect that set a
name said only "is named N", leaving out the supertypes and colors (The Eleventh Hour's
"legendary"), and "exile it with N counters on it" (and "put A and B counters on it")
dropped the "where X is ..." of a counted number (The Eleventh Doctor's time counters).
"any other target" no longer stands for "any target other than that permanent" (an object
other than the source).

## Parser bugs found

- Deceiver of Form: "you may have creatures you control other than ~ become copies of that
  card until end of turn" read "other than ~" as "other than the card copied", so Deceiver
  became a copy too (CR 707.2). `patterns/r707_becomes_copy.rs`: "other than ~" names the
  source. Test `deceiver_of_form_doesnt_become_a_copy_itself` in
  `crates/mtg-engine/tests/cards/roundtrip_pairs_1.rs` (fails on the old code: Deceiver was
  named Craw Wurm).

Checked in play (tests in the same file, passing on the old code too): Grothama counts
damage a spell dealt to it; Smashing Success makes no Treasure when the artifact is
indestructible; Elven Farsight draws only if the card was revealed; Ravenous Slime gets
counters equal to the dying creature's power; Kindle the Carnage repeats more than once;
two Blazing Effigies chain (the dies trigger's source counts as a permanent named Blazing
Effigy, CR 113.7a); Jace, Mirror Mage keeps the card it draws and reveals; Driftgloom
Coyote checks the exiled creature's power as it last existed. Also checked by reading the
AST: Bringer of the Last Gift (graveyards remembered before the sacrifice), Mysterious
Limousine ("each other card" is other than the one exiled this way), Kaldring (a permanent
spell cast this way enters tapped too), Nahiri (free only for Equipment spells), Mystic
Reflection and Storyweave (one use for all the objects entering at once,
`replacement.rs`), Vexing Devil (each opponent in turn order may, its ruling), Sin Prodder
and Distant Memories (until one opponent does), Turncoat Kunoichi (the sneak branch exiles
for good), Mana Cache and Well of Knowledge ("their turn" is the activator's, tested in
`activation_restrictions.rs`).

## Remaining

Of the 151 cards in the taken clusters, these still mismatch; each was checked against the
AST and plays as the text says (wording only) unless noted:

- Primo, the Unbounded: "a number of +1/+1 counters equal to the damage dealt" (rendered
  "that many").
- The Enigma Jewel // Locus of Enlightenment: the craft materials, as before.
- Portal Manipulator: "Those creatures are now attacking that player" after "choose target
  player and any number of target attacking creatures their opponents control" (the
  targets' wording).
- Crystalline Giant: the random missing kind of counter, as before.
- Lethal Exploit: "It gets an additional -1/-1 until end of turn for each modified creature
  you controlled as you cast ~" (rendered as one -X/-X change).
- Lifestream's Blessing: "you gain twice X life" (the value said again).
- Dralnu's Pet: "it enters with flying and with X +1/+1 counters on it" (two abilities).
- Urabrask // The Great Work: chapter III's duration placement and "a graveyard";
  chapter I's "they control".
- Whispersteel Dagger: "this turn" before the mana-spending term.
- Draugr Necromancer: "spells from among cards in exile your opponents own with ice
  counters on them" (quality order).
- Nashi, Searcher in the Dark; Blex, Vexing Pest // Search for Blex: as in roundtrip-tail-6.
- Jace, Mirror Mage: "Draw a card and reveal it" (the reveal step's wording).
- Jetfire, Ooze Flux, Simic Manipulator, Arcee: "Remove one or more +1/+1 counters" is
  compiled as X counters with "X can't be 0", and the amount as X.
- Pia Nalaar, Chief Mechanic; Localized Destruction: "where X is the amount of {E} paid
  this way" / "with power equal to the amount of {E} paid this way" (X unnamed).
- Loamcrafter Faun, Specter of Mortality: "you may discard one or more land cards. When you
  do, ...": compiled as any number, so accepting and choosing none makes the reflexive
  trigger do nothing (not observable).
- Michelangelo, On the Scene; Batroc the Leaper: "a +1/+1 counter on him for each ..." and
  "each of up to X targets".
- Stenn, Paranoid Partisan: "Spells you cast of the chosen type" (word order).
- Eye of Yawgmoth: "Put one into your hand and exile the rest".
- The Wedding of River Song: "target opponent does the same" and "Cards exiled this way that
  don't have suspend gain suspend".
- Turncoat Kunoichi: "instead exile the chosen creature"; Driftgloom Coyote: "If that
  creature had power 2 or less"; Chocobo Kick: "the creature you control deals twice that
  much damage instead".
- Ravenloft Adventurer: "instead exile it and put a hit counter on it" and "cards they own
  in exile with a hit counter on it" (order); Ravenous Slime: "a number of +1/+1 counters
  equal to that creature's power".
- Blazing Effigy: "other sources named ~" is compiled as other permanents named ~ (the
  dies trigger's source is the Effigy as it last existed on the battlefield, so the chain
  works, tested).
- Throne of Empires: "Create five of those tokens instead"; Scepter of Empires: "It deals 3
  damage instead".
- Nexus Mentality: "Draw a card for each counter removed this way".
- Rayami, First of the Fallen: "an exiled creature card with a blood counter on it".
