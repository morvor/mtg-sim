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
