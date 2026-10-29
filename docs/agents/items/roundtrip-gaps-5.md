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
