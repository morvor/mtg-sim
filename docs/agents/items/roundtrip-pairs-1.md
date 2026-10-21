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
